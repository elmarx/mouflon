#!/usr/bin/env -S deno run --unstable --allow-net --allow-read --allow-env --allow-write --allow-run

import { Application, Router } from "https://deno.land/x/oak@v9.0.1/mod.ts";
import addSeconds from "https://deno.land/x/date_fns@v2.22.1/addSeconds/index.ts";
import parseIso from "https://deno.land/x/date_fns@v2.22.1/parseISO/index.js";

import { createHash } from "https://deno.land/std@0.82.0/hash/mod.ts";
import { ensureDir, exists } from "https://deno.land/std@0.114.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.114.0/path/mod.ts";
import { assert } from "https://deno.land/std@0.114.0/testing/asserts.ts";
import { deferred } from "https://deno.land/std@0.114.0/async/deferred.ts";
import { parse } from "https://deno.land/std@0.114.0/flags/mod.ts";
import { passwordGenerator } from "https://deno.land/x/password_generator/mod.ts";
import { encode as encodeBase64Url } from "https://deno.land/std@0.114.0/encoding/base64url.ts";

const MOUFLON_PORT = parseInt(Deno.env.get("MOUFLON_PORT") || "4800");

const VALID_GRACE_SECONDS = 10;

type KeycloakClientConfig = {
  resource: string;
  credentials?: { secret: string };
  realm: string;
  "auth-server-url": string;
  "public-client"?: boolean;
};

type ClientConfig = {
  tokenEndpoint: string;
  authorizationEndpoint: string;
  clientId: string;
  clientSecret?: string;
};

/**
 * turn a simple object to an urlencoded string, i.e.
 * the x=1&y=2 thingy
 */
function toUrlEncodedString(
  p: Record<string, string | undefined | null>,
): string {
  return Object.entries(p)
    .map(([n, v]) => {
      if (v) return `${n}=${v}`;
      else return null;
    })
    .filter((v) => v !== null)
    .join("&");
}

/**
 * build the challenge for a given verifier, i.e.: base64url-encoded sha256 hash
 */
function verifierToChallenge(verifier: string) {
  const hasher = createHash("sha256");
  hasher.update(verifier);

  const hash = new Uint8Array(hasher.digest());
  return encodeBase64Url(hash);
}

const browserApplications = {
  linux: "sensible-browser",
  darwin: "open",
  windows: "explorer",
};
async function openBrowser(url: string): Promise<void> {
  const cmd = browserApplications[Deno.build.os];

  const process = Deno.run({ cmd: [cmd, url] });
  const status = await process.status();
  if (!status.success) {
    console.log("Please open " + url);
  }
}

async function initConfig(config?: string): Promise<ClientConfig> {
  const HOME = Deno.env.get("HOME");
  assert(HOME, "$HOME not set");
  // cache directory according to https://specifications.freedesktop.org/basedir-spec/basedir-spec-0.6.html
  const XDG_CONFIG_HOME = Deno.env.get("XDG_CONFIG_HOME") ||
    join(HOME!, ".config");
  const configDirectory = join(XDG_CONFIG_HOME, "mouflon");

  await ensureDir(configDirectory);

  const configFile = join(configDirectory, `${config}.json`);

  if (!await exists(configFile)) {
    throw new Error(`config file ${configFile} does not exist`);
  }
  const kcConfig = JSON.parse(
    await Deno.readTextFile(configFile),
  ) as KeycloakClientConfig;

  // fetch the openid configuration from the discovery endpoint
  const response = await fetch(
    `${
      kcConfig["auth-server-url"]
    }realms/${kcConfig.realm}/.well-known/openid-configuration`,
  );
  assert(response.ok, response.statusText);
  const oidcConfig = await response.json() as {
    token_endpoint: string;
    authorization_endpoint: string;
  };

  const clientSecret = kcConfig.credentials?.secret;

  assert(
    clientSecret || kcConfig["public-client"] === true,
    "the client either needs to be public or requires a secret",
  );

  return {
    tokenEndpoint: oidcConfig.token_endpoint,
    authorizationEndpoint: oidcConfig.authorization_endpoint,
    clientId: kcConfig.resource,
    clientSecret,
  };
}

/**
 * initialize cache directory where we'll store access-tokens and refresh-tokens
 */
async function initCacheDirectory(): Promise<string> {
  const HOME = Deno.env.get("HOME");
  assert(HOME, "$HOME not set");
  // cache directory according to https://specifications.freedesktop.org/basedir-spec/basedir-spec-0.6.html
  const XDG_CACHE_HOME = Deno.env.get("XDG_CACHE_HOME") ||
    join(HOME!, ".cache");
  const cacheDirectory = join(XDG_CACHE_HOME, "mouflon");

  await ensureDir(cacheDirectory);

  return cacheDirectory;
}

type AccessTokenResponse = {
  access_token: string;
  expires_in: number;
  refresh_expires_in: 1800;
  refresh_token: string;
  token_type: string;
  "not-before-policy": number;
  session_state: string;
  scope: string;
};

type AuthorizationData = {
  // "client side" iat, as the server-time might differ, and so we do not need to read the JWT
  iat: Date;
  atResponse: AccessTokenResponse;
};

/**
 * get an access token response (i.e.: access token and refresh token) by starting a local webserver and letting the user
 * do the "oauth-dance"
 *
 * @param clientConfig
 */
async function fetchAtAuthorizationCodeFlow(
  clientConfig: ClientConfig,
): Promise<AccessTokenResponse | Error> {
  const redirectPath = "/";
  const storedState = passwordGenerator("aA0", 16);
  const codeVerifier = passwordGenerator("aA0", 64);

  const params = {
    client_id: clientConfig.clientId,
    redirect_uri: `http://localhost:${MOUFLON_PORT}${redirectPath}`,
    response_type: "code",
    code_challenge: verifierToChallenge(codeVerifier),
    code_challenge_method: "S256",
    state: storedState,
  };

  const authUrl = `${clientConfig.authorizationEndpoint}?` +
    toUrlEncodedString(params);

  const result = deferred<AccessTokenResponse | Error>();

  const controller = new AbortController();
  const router = new Router();
  router.get(redirectPath, async (ctx) => {
    const code = ctx.request.url.searchParams.get("code");
    const returnedState = ctx.request.url.searchParams.get("state");

    // ideally, these params should be null
    const error = ctx.request.url.searchParams.get("error");
    const errorDescription = ctx.request.url.searchParams.get(
      "error_description",
    );

    if (error) {
      ctx.response.body =
        `Request failed: ${error}. You may now close the browser nevertheless.`;
      result.resolve(new Error(`error=${error}: ${errorDescription}`));
    } else if (returnedState !== storedState) {
      ctx.response.body =
        "State parameter verification failed. You may now close the browser nevertheless.";

      result.resolve(
        new Error(
          `State parameter verification failed: "${returnedState}" != "${storedState}".`,
        ),
      );
    } else {
      const params = {
        grant_type: "authorization_code",
        client_id: clientConfig.clientId,
        client_secret: clientConfig.clientSecret,
        redirect_uri: `http://localhost:${MOUFLON_PORT}${redirectPath}`,
        code,
        // PKCE: now send the original verifier, to proof we know the response to the original challenge (which is a hash)
        code_verifier: codeVerifier,
      };

      const response = await fetch(
        clientConfig.tokenEndpoint,
        {
          body: toUrlEncodedString(params),
          method: "POST",
          headers: {
            "Content-Type": "application/x-www-form-urlencoded",
          },
        },
      );

      if (response.ok) {
        const atResponse = await response.json();

        ctx.response.body = "OK, you may now close the browser.";
        result.resolve(atResponse);
      } else {
        ctx.response.body =
          "Failed, but you may now close the browser nevertheless.";
        // return an error instead of `reject()`, because rejecting here is like throw.
        // I'm not sure if I agree to that handling (of promise/deferred), but OK, this is my workaround,
        // so we can properly clean up.
        result.resolve(
          new Error(JSON.stringify(await response.json(), null, 2)),
        );
      }
    }

    controller.abort();
  });

  const app = new Application();

  app.use(router.routes());
  app.use(router.allowedMethods());

  await openBrowser(authUrl);

  await app.listen({ port: MOUFLON_PORT, signal: controller.signal });

  return result;
}

/**
 * fetch new access token with a given refresh token
 *
 * @param clientConfig
 * @param refreshToken
 */
async function fetchAtRefreshTokenFlow(
  clientConfig: ClientConfig,
  refreshToken: string,
) {
  const response = await fetch(
    clientConfig.tokenEndpoint,
    {
      body:
        `grant_type=refresh_token&client_id=${clientConfig.clientId}&client_secret=${clientConfig.clientSecret}&refresh_token=${refreshToken}`,
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
    },
  );

  return await response.json();
}

async function writeAccessTokenResponse(
  cacheDir: string,
  configName: string,
  atResponse: AccessTokenResponse,
): Promise<void> {
  const auth: AuthorizationData = {
    iat: new Date(),
    atResponse,
  };

  await Deno.writeTextFile(
    join(cacheDir, `${configName}.json`),
    JSON.stringify(
      auth,
      null,
      2,
    ),
  );
}

async function readCachedAuth(
  cacheDir: string,
  configName: string,
): Promise<AuthorizationData | null> {
  const f = join(cacheDir, `${configName}.json`);
  if (!await exists(f)) {
    return null;
  }

  const data = JSON.parse(await Deno.readTextFile(f));
  // "revive" the date object:
  data.iat = parseIso(data.iat, {});

  return data as AuthorizationData;
}

function isAtValid(auth: AuthorizationData) {
  return new Date() <
    addSeconds(auth.iat, auth.atResponse.expires_in - VALID_GRACE_SECONDS);
}

function isRtValid(auth: AuthorizationData) {
  return new Date() <
    addSeconds(
      auth.iat,
      auth.atResponse.refresh_expires_in - VALID_GRACE_SECONDS,
    );
}

async function getAccessToken(
  cacheDir: string,
  configName: string,
  clientConfig: ClientConfig,
): Promise<string> {
  const cachedAuth = await readCachedAuth(cacheDir, configName);

  if (cachedAuth) {
    if (isAtValid(cachedAuth)) {
      return cachedAuth.atResponse.access_token;
    } // access token expired, but refresh token still valid
    else if (isRtValid(cachedAuth)) {
      const rtResponse = await fetchAtRefreshTokenFlow(
        clientConfig,
        cachedAuth.atResponse.refresh_token,
      );

      await writeAccessTokenResponse(cacheDir, configName, rtResponse);
      return rtResponse.access_token;
    }
  }

  // no token at all, or expired token, do the normal authorization code flow
  const atr = await fetchAtAuthorizationCodeFlow(clientConfig);
  if (atr instanceof Error) throw atr;
  await writeAccessTokenResponse(cacheDir, configName, atr);
  return atr.access_token;
}

async function main() {
  const cacheDir = await initCacheDirectory();

  const args = parse(Deno.args);
  const [config = "default"] = args._;

  const clientConfig = await initConfig(config as string);

  const at = await getAccessToken(cacheDir, config as string, clientConfig);

  console.log(at);
}

if (import.meta.main) {
  await main();
}
