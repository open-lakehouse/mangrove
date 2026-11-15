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

### Authorization

By default every request is allowed (`policy.kind: constant`). With the
`olai-auth` feature and `auth.mode: olai-auth`, a Cedar policy decision point
can decide instead:

```yaml
policy:
  kind: authorizer
  url: https://authz.example.com      # olai-auth's PDP wire API: {url}/v1/authorize
  token: { env: UC_PDP_TOKEN }        # this server's secret for the decision point
  cache-ttl-secs: 5                   # reuse decisions (default 5, 0 = off)
```

Each check becomes a Cedar request in the `UnityCatalog` namespace: the
verified caller as `Lakehouse::User` or `Lakehouse::ServiceAccount`, one action
per API operation (`GetTable`, `CreateTable`, `GenerateTemporaryTableCredentials`,
… generated from the protos, plus the Delta API's `CommitTable`, `RenameTable`,
`AdoptStagingTable`, `GenerateTemporaryStagingTableCredentials` and the storage
proxy's `ReadData` / `WriteData`), and the securable by its full name, e.g.
`UnityCatalog::Table::"main.sales.orders"`. The securable and each ancestor
(schema, catalog, metastore) go along as entities, each with its ancestors as
parents and typed attributes (`resource.schema`, `resource.catalog`). The
decision point's entity loader adds what the caller holds on each, from the
grants: `privileges` (granted on that securable, `OWNER` for its owner) and
`inherited_privileges` (also from its ancestors). Request facts go in the
context: `credential_operation` (`READ` / `READ_WRITE`) on every vend, `url`
on path vends, `is_creator` on adopting a staging table. What an operation
requires is the policy's business. Anything it can't decide, and any request
without a verified caller, is denied. The schema is published as
`policy::CEDAR_SCHEMA_FRAGMENTS` (entities in `cedar/UnityCatalog.cedarschema`,
the generated actions, `cedar/DeltaApi.cedarschema`,
`cedar/StorageProxy.cedarschema`), for a host to compose with its `Lakehouse`
fragment. With the default `constant` policy everything stays allowed.

Grants are kept through the grants API (`GET` / `PATCH
/permissions/{securable_type}/{full_name}`, the Unity Catalog OSS shape;
privileges in either spelling, `USE CATALOG` or `USE_CATALOG`). The creator of
a securable owns it, an `owner` update moves ownership, and deletes and renames
take the grants inside along. With `auth.mode: olai-auth` grants live in the
`authz` schema of the identity database (`auth.olai.database-url`), where the
decision point's loader reads them; otherwise in memory.

### Trusted vending for Neon object storage

Neon object storage speaks S3 but has no STS and no bucket- or prefix-scoped
keys. With the `neon-trusted-vending` feature and `vending.neon.enabled`, the
server mints branch-scoped, expiring keys through Neon's control plane. Such a
key reaches **every bucket on the branch**, and on the branch's descendants, at
the granted access level until it expires. So the vendor is *trusted*: enable
it only behind real authentication and authorization.

A Neon credential stays within the UC credentials API. It is an `aws_iam_role`
whose `role_arn` is a Neon ARN, with no keys:

```yaml
# POST /credentials
name: neon_branch
purpose: STORAGE
aws_iam_role:
  role_arn: arn:neon:storage::<project_id>:branch/<branch_id>
```

Leave out either id (`arn:neon:storage::` leaves both out) and the server fills
it in from `vending.neon` before storing. Ids in the ARN always win. External
locations over `s3://<bucket>` then use the credential like any other.

```yaml
vending:
  neon:
    enabled: false                     # default; Neon credentials register but don't vend
    project-id: falling-sun-12345678   # required when enabled
    # branch-id: { env: NEON_BRANCH_ID }  # default; set for Neon Functions
    # api-key: { env: NEON_API_KEY }      # default; never stored with a credential
    ttl-secs: 900                      # 900 (15 min) to 43200 (12 h); default 3600
```

Neon Functions get `NEON_BRANCH_ID` but not `NEON_PROJECT_ID`, so `project-id`
is required configuration.

With `enabled: true` the server refuses to start unless `auth.mode: olai-auth`
**and** `policy.kind: authorizer` are set. Trusted keys reach a whole branch, so
they must never be one anonymous or allow-all request away.

- **The TTL and `enabled` are the controls.** Neon enforces a key's expiry to
  the second, but revocation is eventually consistent. In testing, a revoked key
  was accepted again about 13 s later and rejected for good only after about
  20 s. Revoking a key doesn't stop it promptly, so keep the TTL short. To stop
  vending, set `enabled: false`.
- The minter reuses one key per access level until half its lifetime has
  passed, and revokes its own expired `olai-vend-*` keys.
- Delta API credentials carry `s3.endpoint`, `s3.region` and
  `s3.path-style-access`. UC REST credentials have no endpoint field, so those
  clients set `AWS_ENDPOINT_URL_S3` to the branch's storage endpoint.
- Without the feature, or with `enabled: false`, vending against a Neon
  credential answers `400 FAILED_PRECONDITION` with the reason.

### Vend audit log

Every credential vend handed to a client, on UC REST (`temporary-*-credentials`)
and the Delta API (`/credentials`, staging tables), for every credential kind,
emits one `tracing` event at target `unitycatalog::audit` (`event =
"credential_vend"`). The event records:

- `outcome`: `ok`, or `error` with an `error_code`
- `surface`: `uc_rest` or `delta_api`
- `principal`
- `securable`: the full name, or the path for path vends
- `location`
- `access`: `read` or `read_write`
- `credential_kind`: `neon_storage`, `aws_iam_role`, `local`, …
- `credential_id`: the first 8 characters of the S3 access key id, or empty
- `expires_at`
- `policies`: the ids of the Cedar policies that allowed the vend's checks,
  under `policy.kind: authorizer`

Secrets are never logged. The storage proxy's internal vends, whose keys never
leave the server, aren't audited.

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
| `olai-auth` |       | `auth.mode: olai-auth`: Neon Auth JWTs over a JWKS URL and service-account API keys, and `policy.kind: authorizer` (implies `embed`). |
| `neon-trusted-vending` | | Trusted vending for Neon object storage (`vending.neon`). Not in `default` or `bin`: a host opts in. |
| `bin`     |         | The deployable `uc-server` binary: everything above except `neon-trusted-vending`, plus the CLI parser and the healthcheck client. |

A config that asks for a backend or surface the build doesn't include fails at
startup.

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
