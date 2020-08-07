# 🐑 Mouflon — CLI tool to get OIDC tokens

<img align="right" src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/e5/Mouflon_Corse.jpg/300px-Mouflon_Corse.jpg" />

Mouflon acts as an *OIDC client* to retrieve an **access token** from an OIDC provider.

Upon initial execution, it opens a browser and executes the typical OIDC redirects to get an *access token*
via *authorization_code* grant.

If successful, it caches the *access token response* (thus both the *access token* and the *refresh token*), and then
returns the *access token* (as long as it's valid), or uses the *refresh token* to refresh the *access token* 
and of course return the new *access token*. If also the *refresh token* is expired, it again opens the browser to execute
the OIDC authorization.

## Status

*mouflon* works, but is pretty basic and not very flexible.

- opening the browser works only in Linux and the fallback solution is implemented very naively
- supports only keycloak, only a single realm and a single client
- close to no error handling. So it will throw stack traces without any hints upon errors

Of course all of these are possible future improvements :)  

## Installation

Mouflon uses [*deno*](https://deno.land/), so make sure to have it [installed](https://deno.land/#installation).

Place file `mouflon.ts` into your `$PATH` (e.g. `~/bin`) and set the execution-bit (e.g. `chmod +x mouflon.ts`).

## Configuration 

### Keycloak

Create an OIDC client (Standard flow enabled), should be "confidential", allow `http://localhost:3000/` as redirect URL.

Download the "*Keycloak OIDC JSON*" file available under the "*Installation*" tab.

### Mouflon

Copy said JSON-file into `~/.config/mouflon/default.json` (if you set `$XDG_CONFIG_HOME` replace `~/.config` with that value).

Future versions could allow other configurations (selectable via CLI-arg) and other providers.

Currently, *mouflon* does **not** validate the JSON file.

## Usage

Simply execute `mouflon.ts` or `./mouflon.ts`

## Examples

for bash

```shell script
curl -H "Authorization: Bearer $(mouflon.ts)" https://example.com/protected
```
    
or fish shell

```shell script
AT=(mouflon.ts) curl -H "Authorization: Bearer $AT" https://example.com/protected
```
