# olai-uc-server

**Unity Catalog REST server with pluggable storage backends — a
deployable catalog service and the library it is built from.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_server` (imports and paths use that
> name); the crate is published to crates.io as `olai-uc-server` while the naming
> settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

This crate is two things at once:

- A **library** (`unitycatalog_server`) providing the Unity Catalog REST
  handlers, routing, storage services, and the reusable handler patterns
  (in-memory backend, proxy/federation connectors) that a custom deployment can
  build on.
- A **deployable binary**, `uc-server` (gated behind the `bin` feature), with
  `serve` / `migrate` / `healthcheck` subcommands. This is the same binary that
  ships as the [`mangrove`](https://github.com/open-lakehouse/mangrove) Docker
  image (`ghcr.io/open-lakehouse/mangrove`) with the web UI bundled in.

The `uc-server` binary is distinct from the `uc` client CLI (crate
`olai-uc-cli`): `uc-server` *runs* a catalog, `uc` *talks to* one.

## Install the server binary

The binary is gated behind the `bin` feature (off by default, so library
consumers don't pull the serve/store/CLI stack), so enable it when installing:

```sh
cargo install olai-uc-server --features bin
```

This installs `uc-server`:

```sh
uc-server serve            # run the catalog server (REST + bundled UI)
uc-server migrate          # run database migrations
uc-server healthcheck      # probe /health (used by the Docker HEALTHCHECK)
```

`serve` loads a YAML config file (also `UC_SERVER_CONFIG`), overlaid by CLI flags
(`--host` / `--port` / `--no-ui`, CLI winning).

Prefer a container? Pull the image instead:

```sh
docker run -p 8080:8080 ghcr.io/open-lakehouse/mangrove:latest
```

## Use as a library

```toml
[dependencies]
# The `package` rename keeps the `unitycatalog_server` import path.
unitycatalog-server = { package = "olai-uc-server", version = "0.0.2" }
```

A bare library build brings in only the handler/routing surface (default
features `axum` + `memory`); the `bin` feature and its storage-backend /
CLI / UI-serving dependencies stay out unless you opt in.

## Embed without a socket

`embed::start` builds the same service `uc-server serve` runs, but returns the
axum `Router` instead of binding an address. That suits hosts that receive
requests some other way, like a Node-API addon behind a Neon Function's fetch
handler:

```toml
unitycatalog-server = { package = "olai-uc-server", version = "…", default-features = false, features = ["embed", "postgres"] }
```

```rust,ignore
use unitycatalog_server::{config::Config, embed};

let service = embed::start(&Config::from_env()?).await?;
// per request: service.router.clone().oneshot(request).await
```

`Config::from_env` reads the configuration from `UC_SERVER_CONFIG_YAML` (inline
YAML) or the file named by `UC_SERVER_CONFIG`. When `DATABASE_URL` is set it
switches the backend to Postgres at that URL, and `DATABASE_URL_UNPOOLED`, if
set, becomes the migration URL. Those are the variables Neon sets for a branch.

Like `serve`, `start` never migrates a durable backend, and it fails unless the
schema matches this build's migrations. Run `embed::migrate` (or
`uc-server migrate`) out of band first.

For a server that's reachable directly, use bearer authentication:

```yaml
ui:
  serve: false
auth:
  mode: bearer
  tokens:
    - principal: admin
      token: { env: UC_ADMIN_TOKEN }
```

With the `olai-auth` feature, `auth.mode: olai-auth` verifies callers with
[`olai-auth`](https://crates.io/crates/olai-auth) instead. A caller presents an
EdDSA JWT of the configured issuer (Neon Auth) or an `olk_` API key of a
service account (`authz.api_keys`). Whether a user is an admin or banned comes
from Neon Auth's `neon_auth."user"`, never from the token. The policy and the
Delta API backend see the verified caller as `RequestContext::verified_principal`.

```yaml
auth:
  mode: olai-auth
  olai:
    issuer: { env: NEON_AUTH_BASE_URL }     # the exact `iss` of OAuth access tokens
    jwks-url: { env: NEON_AUTH_JWKS_URL }
    audiences: [https://unitycatalog.example.com]
    accept-session-tokens: true             # also the web app's session tokens (iss = aud = the Auth URL's origin)
    database-url: { env: AUTH_DATABASE_URL } # the identity tables; defaults to the Postgres backend
    resource: https://unitycatalog.example.com
    resource-metadata-url: https://unitycatalog.example.com/.well-known/oauth-protected-resource
```

The identity tables are read through their own small pool, so they can live in
another database than the catalog. On Neon that's the branch's `neondb`, while
the catalog has a database of its own. A request with no token or a rejected one
gets a 401 with an RFC 6750 challenge; a banned user gets a 403. With `resource`
set, `/.well-known/oauth-protected-resource` is served without authentication.
GitHub Actions OIDC tokens aren't accepted yet.

## Feature flags

| Feature   | Default | What it enables                                                                          |
| --------- | :-----: | ---------------------------------------------------------------------------------------- |
| `axum`    |   yes   | The axum router, extractors, and served HTTP surface.                                    |
| `memory`  |   yes   | The in-memory handler backend.                                                           |
| `proxy`   |         | Unity Catalog client handlers to wrap in `handlers::Authorized` as proxy leaves, and upstream `routing`. |
| `federation` |      | Federation over the proxy connector (implies `proxy`).                                   |
| `embed`   |         | Config loading and the socket-less server (`embed::start`, `run::serve`, `run::migrate`). Needs a backend feature to serve anything. |
| `postgres` |        | The Postgres backend (implies `embed`).                                                  |
| `sqlite`  |         | The SQLite backend, including the default ephemeral `:memory:` one (implies `embed`).    |
| `swagger` |         | Swagger UI for the bundled OpenAPI specs.                                                |
| `ui`      |         | Serving the bundled single-page app from `./web` (`ui.serve`).                           |
| `storage-proxy` |   | The same-origin storage byte-proxy (`storage_proxy.enabled`).                            |
| `files`   |         | The embedded Files API (`files.enabled`).                                                |
| `olai-auth` |       | `auth.mode: olai-auth`: Neon Auth JWTs over a JWKS URL and service-account API keys (implies `embed`). |
| `bin`     |         | The deployable `uc-server` binary: all of the above, plus the CLI parser and the healthcheck client. |

A config that asks for a backend or surface the build doesn't include fails at
startup.

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
