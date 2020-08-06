#!/usr/bin/env -S deno run --unstable --allow-net --allow-read

import { Application, Router } from "https://deno.land/x/oak@v6.0.1/mod.ts";
import {
  viewEngine,
  engineFactory,
  adapterFactory,
} from "https://deno.land/x/view_engine@v1.3.0/mod.ts";
import Random from "https://deno.land/x/random/Random.js";
import { readJson } from "https://deno.land/std@v0.61.0/fs/mod.ts";
import { assert } from "https://deno.land/std@v0.61.0/testing/asserts.ts";

declare module "https://deno.land/x/oak@v6.0.1/mod.ts" {
  interface Context {
    render: (fileName: string, data?: object) => void;
  }

  interface RouterContext {
    render: (fileName: string, data?: object) => void;
  }
}

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

async function readKeycloakConfig(
  configFile: string = "./keycloak.json",
): Promise<ClientConfig> {
  const kcConfig = await readJson(configFile) as KeycloakClientConfig;

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

async function main() {
  const config = await readKeycloakConfig();
  const authUrl =
    `${config.authorizationEndpoint}?client_id=${config.clientId}&redirect_uri=http://localhost:3000/redirect&response_type=code`;

  const r = new Random();

  const controller = new AbortController();

  const router = new Router();

  router.get("/", async (ctx) => {
    ctx.render(
      "index.html",
      {
        data: {
          nonce: r.string(12),
          authUrl,
        },
      },
    );
  }).get("/redirect", async (ctx) => {
    const code = ctx.request.url.searchParams.get("code");

    const response = await fetch(
      config.tokenEndpoint,
      {
        body:
          `grant_type=authorization_code&client_id=${config.clientId}&client_secret=${config.clientSecret}&code=${code}&redirect_uri=http://localhost:3000/redirect`,
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
      },
    );

    const token = await response.json();

    ctx.response.body = "OK, you may close the browser";
    console.log(JSON.stringify(token, null, 2));
    controller.abort();
  });

  const denjuckEngine = engineFactory.getDenjuckEngine();
  const oakAdapter = adapterFactory.getOakAdapter();

  const app = new Application();

  app.use(viewEngine(oakAdapter, denjuckEngine));
  app.use(router.routes());
  app.use(router.allowedMethods());

  console.log("Please open " + authUrl);

  await app.listen({ port: 3000, signal: controller.signal });
}

if (import.meta.main) {
  await main();
}
