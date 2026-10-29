# olai-sharing-server

A composable Open Sharing / Delta Sharing server, built on the protocol
implementation in [`olai-sharing-api`](../sharing-api). See
[`ARCHITECTURE.md`](../sharing-api/ARCHITECTURE.md) for the design.

- **Shares map onto a source catalog.** A share is a list of share objects
  (`TABLE`, `SCHEMA`, `VOLUME`, `AGENT_SKILL`) naming assets in a catalog source:
  a static list in the config, a Unity Catalog REST server (this repository's
  `uc-server`, UC OSS or Databricks; feature `unity`), or your own
  `CatalogSource`. Locations and formats are
  resolved from the source at request time.
- **Storage access is delegated or local.** A *delegated* source vends
  short-lived, downscoped credentials, so the server stores no storage secrets.
  A *local* source uses credentials stored in the server, sealed, and vended by
  the same `olai-uc-credentials` code the UC server uses. Either way, query
  responses carry presigned URLs that never outlive the credential, and the
  Open Sharing `temporary-*-credentials` endpoints hand the credential on.
- **Recipients use bearer tokens.** Creating a recipient issues a token with a
  one-time activation link. Opening the link returns the `.share` profile, and
  from then on the server keeps only the token's sha256 digest.
- **Secrets are sealed at rest.** Envelope encryption with pluggable
  key-encryption keys, rotated lazily on read or eagerly with `rotate-keys`.

## Surfaces

| Path | What |
|---|---|
| `/api/v1/delta-sharing/…` | Delta Sharing protocol (recipient token) |
| `/api/1.0/opensharing/…` | Open Sharing protocol: adds volumes, agent skills, temporary credentials (recipient token) |
| `/api/1.0/opensharing/activations/{code}` | One-time activation link, returns the sharing profile |
| `/api/2.1/unity-catalog/…` | Provider admin API (feature `unity`): `shares`, `shares/{name}/permissions`, `recipients`, `recipients/{name}/rotate-token`, `credentials`, `external-locations` (provider token) |
| `/health` | Liveness |

## Features

| Feature | Default | What |
|---|---|---|
| `postgres` | ✓ | The Postgres `ShareStore` (sqlx, embedded migrations) |
| `unity` | ✓ | Unity Catalog integration: the UC REST catalog source and the UC-shaped provider admin API |
| `bin` | | The `sharing-server` binary (implies `postgres` and `unity`) |

### Without Unity Catalog

With `default-features = false` (optionally plus `postgres`) the crate uses no
catalog's models: the store, recipient tokens and activation, sealed secrets,
the static source and storage access remain. Credential vending and object
stores come from `olai-uc-credentials` / `olai-uc-object-store`, over their own
credential types. A catalog that already has discovery and an admin surface
implements `CatalogSource` (or the `olai-sharing-api` ports directly),
provisions shares and recipients through the `ShareStore`
(`recipients::issue_token` issues tokens), and serves `embed::router`. See
`tests/custom_mount.rs`.

## Running

```sh
cargo run -p olai-sharing-server --features bin -- -c config.yaml migrate
cargo run -p olai-sharing-server --features bin -- -c config.yaml serve
```

Without a config file the server runs on an in-memory store with an insecure
development key. Below is a configuration with a Unity Catalog source and
Postgres (for example Neon). The direct URL is used for migrations and the
startup schema check, which need a session-level advisory lock:

```yaml
public_url: https://sharing.example.com
database:
  type: postgres
  url: { env: DATABASE_URL }                 # pooled is fine
  migration_url: { env: DATABASE_URL_UNPOOLED }
encryption:
  active: { id: v1, key: { env: SHARING_KEK_V1 } }   # 32 bytes, base64
provider_tokens: [{ env: SHARING_PROVIDER_TOKEN }]
sources:
  - name: uc
    type: unity
    url: https://uc.example.com/api/2.1/unity-catalog/
    token: { env: UC_TOKEN }
    access: delegated
```

Give the server its own database: its migrator keeps its own
`_sqlx_migrations` table.

### Sharing a table

```sh
H='Authorization: Bearer <provider token>'; U=https://sharing.example.com/api/2.1/unity-catalog
curl -X POST  $U/shares -H "$H" -d '{"name":"sales"}'
curl -X PATCH $U/shares/sales -H "$H" -d '{"updates":[{"action":"ADD","data_object":
  {"name":"main.sales.orders","data_object_type":"TABLE","history_data_sharing_status":"ENABLED"}}]}'
curl -X POST  $U/recipients -H "$H" -d '{"name":"acme","authentication_type":"TOKEN"}'
curl -X PATCH $U/shares/sales/permissions -H "$H" -d '{"changes":[{"principal":"acme","add":["SELECT"]}]}'
```

Send the recipient's `tokens[0].activation_url`. Opening it yields a profile
that the `delta-sharing` clients read directly.

## Testing

`cargo test -p olai-sharing-server` runs the end-to-end suite on the
in-memory store; `--no-default-features` runs the catalog-free host test
(`tests/custom_mount.rs`). The Postgres store test runs when
`SHARING_SERVER_TEST_DATABASE_URL` points at a disposable database (see
`tests/postgres.rs`).
