#!/usr/bin/env -S deno run --unstable --allow-net --allow-read --allow-env --allow-write --allow-run

import { Application, Router } from "https://deno.land/x/oak@v6.3.2/mod.ts";
import addSeconds from "https://deno.land/x/date_fns@v2.15.0/addSeconds/index.js";
import parseIso from "https://deno.land/x/date_fns@v2.15.0/parseISO/index.js";

import { ensureDir, exists } from "https://deno.land/std@0.79.0/fs/mod.ts";
import { join } from "https://deno.land/std@0.79.0/path/mod.ts";
import { assert } from "https://deno.land/std@0.79.0/testing/asserts.ts";
import { deferred } from "https://deno.land/std@0.79.0/async/deferred.ts";
import { parse } from "https://deno.land/std@0.79.0/flags/mod.ts";

const MOUFLON_PORT = parseInt(Deno.env.get("MOUFLON_PORT") || "4800");

const VALID_GRACE_SECONDS = 10;

type KeycloakClientConfig = {
  resource: string;
  credentials: { secret: string };
  realm: string;
  "auth-server-url": string;
};

type ClientConfig = {
  tokenEndpoint: string;
  authorizationEndpoint: string;
  clientId: string;
  clientSecret: string;
};

async function openBrowser(url: string): Promise<void> {
  // TODO: this is Linux specific, add support for MacOS
  const process = Deno.run({ cmd: ["sensible-browser", url] });
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

  return {
    tokenEndpoint: oidcConfig.token_endpoint,
    authorizationEndpoint: oidcConfig.authorization_endpoint,
    clientId: kcConfig.resource,
    clientSecret: kcConfig.credentials.secret,
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
  const redirectUri = `http://localhost:${MOUFLON_PORT}${redirectPath}`;
  const authUrl =
    `${clientConfig.authorizationEndpoint}?client_id=${clientConfig.clientId}&redirect_uri=${redirectUri}&response_type=code`;

  const result = deferred<AccessTokenResponse | Error>();

  const controller = new AbortController();
  const router = new Router();
  router.get(redirectPath, async (ctx) => {
    const code = ctx.request.url.searchParams.get("code");

    const response = await fetch(
      clientConfig.tokenEndpoint,
      {
        body:
          `grant_type=authorization_code&client_id=${clientConfig.clientId}&client_secret=${clientConfig.clientSecret}&code=${code}&redirect_uri=${redirectUri}`,
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
      result.resolve(new Error(JSON.stringify(await response.json(), null, 2)));
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
  atResponse: AccessTokenResponse,
): Promise<void> {
  const auth: AuthorizationData = {
    iat: new Date(),
    atResponse,
  };

  await Deno.writeTextFile(
    join(cacheDir, "authorizationData.json"),
    JSON.stringify(
      auth,
      null,
      2,
    ),
  );
}

async function readCachedAuth(
  cacheDir: string,
): Promise<AuthorizationData | null> {
  const f = join(cacheDir, "authorizationData.json");
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
  clientConfig: ClientConfig,
): Promise<string> {
  const cachedAuth = await readCachedAuth(cacheDir);

  if (cachedAuth) {
    if (isAtValid(cachedAuth)) {
      return cachedAuth.atResponse.access_token;
    } // access token expired, but refresh token still valid
    else if (isRtValid(cachedAuth)) {
      const rtResponse = await fetchAtRefreshTokenFlow(
        clientConfig,
        cachedAuth.atResponse.refresh_token,
      );

      await writeAccessTokenResponse(cacheDir, rtResponse);
      return rtResponse.access_token;
    }
  }

  // no token at all, or expired token, do the normal authorization code flow
  const atr = await fetchAtAuthorizationCodeFlow(clientConfig);
  if (atr instanceof Error) throw atr;
  await writeAccessTokenResponse(cacheDir, atr);
  return atr.access_token;
}

async function main() {
  const cacheDir = await initCacheDirectory();

  const args = parse(Deno.args);
  const [config = "default"] = args._;

  const clientConfig = await initConfig(config as string);

  const at = await getAccessToken(cacheDir, clientConfig);

  console.log(at);
}

if (import.meta.main) {
  await main();
}
