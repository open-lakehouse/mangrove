# Open Sharing server architecture

Status: accepted (2026-10-29). Delivered as one stacked PR series. The
layers are listed under [Delivery](#delivery).

## Context

This crate already has the beginnings of an Open Sharing / Delta Sharing
surface: a `SharingBackend` port, blanket discovery and query handlers, and an
axum router. Nothing mounts or implements it, though. The only backend is the
in-memory `testing` one, and the query path has gaps:

- It goes through DataFusion and `olai-uc-datafusion`'s `ReconciledLogProvider`.
  That pulls DataFusion and the git-pinned `deltalake-core` into a crate whose
  real job is listing files.
- `/query` returns bare `file` lines. They carry raw storage paths: no
  presigned URL, no protocol or metadata lines, and no stats. Version,
  timestamp, predicate hints and limit hints are all ignored.
- The server never parses `delta-sharing-capabilities`, and
  `DeltaSharingCapabilities::parse_header` panics on malformed input.
- Only `GET /shares` passes pagination to the backend. Schemas are derived from
  tables only.
- There is no recipient authentication and no share store.

The upstream reference implementation,
[OpenSharing-IO/OpenSharing](https://github.com/OpenSharing-IO/OpenSharing), is
Java on Armeria with delta-kernel-java, licensed Apache-2.0. It defines the
OpenSharing spec (`/api/1.0/opensharing`, Share → Schema → Asset, with volumes,
agent skills and models as asset kinds). Its implementation is narrower than
the spec:

| Capability | Spec (OpenSharing ⊇ Delta Sharing) | Upstream (Java) | Here, before this work |
|---|---|---|---|
| List/get shares, schemas, tables, all-tables (paged) | ✓ | ✓ | ✓ (only shares paged) |
| Table version (`startingTimestamp`) | ✓ | ✓ | version only |
| Table metadata (`version`/`timestamp`) | ✓ | ✓ | latest only |
| `POST …/query` (presigned file URLs, hints) | ✓ | ✗ | raw paths, no hints |
| `GET …/changes` (CDF) | ✓ | ✗ | 501 |
| `responseformat=delta` | ✓ | ✓ (negotiated) | ✗ |
| Async query (`/queries/{id}`) | ✓ | ✗ | 501 |
| `temporary-table-credentials` (vending) | ✓ | ✓ | ✗ |
| Volumes / agent skills | ✓ | ✗ | discovery + vend via port |
| Iceberg REST view of tables | ✓ | ✗ | ✗ |
| Recipient bearer tokens + activation | ✓ | ✓ | ✗ |
| Provider admin API (recipients, shares, permissions) | — | ✓ | UC-shaped CRUD in `uc-server` only |
| Pluggable catalog | — | one `CatalogConnector`, YAML only | `SharingBackend` (unimplemented) |

Upstream has no `/query`. Classic `delta-sharing` clients therefore can't
read table data from it: they get metadata and must use vended credentials
themselves. We want both models.

## Decision

Build a **composable sharing server**:

- It maps shares onto assets that already exist in a **pluggable source
  catalog**.
- It serves classic Delta Sharing `/query` with **presigned URLs** and also the
  OpenSharing **credential-vending** endpoints, from one vending implementation.
- It reads Delta through `delta_kernel` directly, with **no DataFusion**.
- It owns its share, recipient and token state in Postgres, and seals secrets
  at rest behind a key-rotation seam.

### Ports

`SharingBackend` is split into narrow ports. They stay generic over the
request context `Cx`, which becomes the authenticated recipient.

| Port | Responsibility | First implementations | Later |
|---|---|---|---|
| `RecipientAuthenticator` | bearer token → `Cx` (recipient, granted shares) | Postgres, sha256-hashed tokens | OIDC / JWT |
| `ShareStore` | shares (UC `shares/v1` `Share` / `DataObject`), recipients, grants, tokens | Postgres (sqlx) | — |
| `CatalogSource` | resolve a `DataObject` (`catalog.schema.table`; `SCHEMA` objects expand) to a `SourceAsset { url, format, id, … }`, paged | UC REST (`olai-uc-client`: our UC or Databricks), static | Iceberg REST, UC-Delta API, in-process UC store |
| `StorageAccess` | the server's own log reads, `presign(url, ttl)`, `vend(url, op)` | delegated (catalog-vended), local (own credentials) | — |
| `TableFormat` | version / metadata / query / changes as protocol actions | Delta (kernel) | Iceberg |
| `KeyProvider` (`olai-store`) | wrap/unwrap data keys for sealed secrets | `LocalKeyProvider` | cloud KMS / Key Vault |

The recipient-facing ports live in `olai-sharing-api`: `ShareCatalog`, which
covers discovery and resolution and is authorized through one `authorize` hook;
`StorageAccess`; and `TableFormat`. A host implements `ShareCatalog` on top of
its `ShareStore` and `CatalogSource`, which belong to the sharing server, and
puts `RecipientAuthenticator` in front of the router.

The wire types (generated request/response models, the NDJSON query lines and
the capabilities header) live in `olai-sharing-common`, which both this crate
and `olai-sharing-client` depend on. The server side never depends on the
client, so the client can take on heavier dependencies (DataFrame libraries and
the like) without the server inheriting them.

A `SharingService` holds one implementation per port, plus a format registry.
It implements the existing handler traits, so discovery, query and asset
serving stay identical whatever backs them. The router mounts the same
handlers twice behind a bearer-auth layer: under `/api/1.0/opensharing` (the
spec, including assets and credential endpoints) and under the Delta Sharing
prefix the `olai-sharing-client` targets.

### Discovery maps shares onto a source catalog

A share is a UC `Share` whose `objects[]` name assets in a source catalog. The
`objects[]` entries are tables, whole schemas, volumes and agent skills. The
sharing server stores the share. It resolves each object through
`CatalogSource` at request time: storage location, format and identity come
from the catalog, which stays the source of truth. `DataObject` fields we
honor:

- `shared_as` renames the object for recipients.
- `start_version` clamps time travel and CDF.
- `history_data_sharing_status` gates version/timestamp reads.
- `partitions` restricts which partitions are shared. This is a follow-up.

### Storage access: delegated or local

Every mode serves the same recipient surface: `/query` with presigned URLs and
the `temporary-{table,volume,skill}-credentials` endpoints.

- **Delegated.** This is the default whenever the source catalog can vend.
  The server stores **no storage secrets**. Per request it asks the catalog
  for short-lived, downscoped credentials, using UC
  `temporary-table-credentials` / `temporary-volume-credentials` (later
  Iceberg REST `loadCredentials`). It caches them for a short TTL and uses
  them for three things:
  - reading the Delta log;
  - presigning file URLs (S3 SigV4 with the session token, or the Azure SAS
    appended to the blob URL);
  - answering the recipient credential endpoints, by passing the credential
    through.

  The URLs and credentials we hand out never outlive what the catalog
  issued. GCS is the exception for presigning: an OAuth access token cannot
  sign URLs, so GCS tables are served via vending only, or the catalog must
  vend signed URLs.
- **Local.** This mode is for standalone deployments, static catalogs, or
  catalogs that can't vend. The server holds storage credentials itself, in
  UC's model: a storage `Credential` (IAM role, Azure service principal or
  managed identity, storage key) plus external locations matched by longest
  URL prefix. It presigns and vends with the **same** code `uc-server` uses,
  extracted into `olai-uc-credentials`, so the workspace has exactly one
  vending implementation.

Each catalog source chooses its mode (`access = delegated | local`).

### Secrets at rest

Storage credentials (local mode) and catalog-source client secrets (PATs,
OAuth client secrets) are sealed with `olai-store`'s `EnvelopeEncryptor`.
This is the same envelope encryption `uc-server` uses. Recipient tokens are
stored only as hashes.

`KeyProvider` is async and keyed by KEK id, so an external key store slots in
without format changes. Rotation is **lazy**. When a read opens a blob sealed
under a retired KEK, the server `rewrap`s it under the active KEK and writes it
back. A `rotate-keys` admin command re-wraps everything eagerly. The first
release ships `LocalKeyProvider` (config-supplied KEKs) and a
`key_provider = local | …` config seam. A cloud KMS provider is a follow-up, and
it is the expected choice for hosted (Neon) deployments: the KEK stays in KMS,
and DEK unwraps are cached in-process.

### Delta without DataFusion

The Delta `TableFormat` uses only `delta_kernel` and its default engine:

- **Version and timestamp.** `Snapshot::builder_for(..).at_version(v)` serves a
  version. Timestamps resolve through
  `history_manager::latest_version_as_of` (`HistoryCommitType::Recreatable`).
- **Files.** A `Scan` with a best-effort kernel `Predicate` translated from
  `jsonPredicateHints` prunes files. Unsupported hint shapes are dropped:
  hints are advisory, and the recipient still filters. Then `scan_metadata` →
  `visit_scan_files` yields `ScanFile { path, size, stats, partition_values, dv_info }`.
- **Response.** The response streams protocol, metaData, then `file` lines.
  Each `file` line has a presigned `url`, a stable `id` (a hash of the path),
  `partitionValues`, `size`, `stats` and `expirationTimestamp`. `limitHint` is
  applied cumulatively over `numRecords` from stats.
- **Capabilities.** `delta-sharing-capabilities` is parsed leniently. The first
  release serves `responseformat=parquet`, and `responseformat=delta` and CDF
  (`table_changes`) are follow-ups.

`ReconciledLogProvider` stays in `olai-uc-datafusion` for the log explorer.
This crate drops `datafusion` and `olai-uc-datafusion`.

### Hosts

`olai-sharing-server` is a library plus binary, structured like Headwaters'
service crate:

- **Startup.** `embed::Service::start` returns a socket-less `Router`.
- **Binary subcommands.** `serve`, `migrate`, `healthcheck` and `rotate-keys`.
- **Configuration.** Config is layered: file, then env, then `DATABASE_URL`.
- **Schema.** sqlx migrations are embedded in the binary. They run only via
  `migrate`, against the unpooled URL, and `serve` refuses to start on schema
  drift.
- **Admin API.** It reuses the UC `shares`, `recipients` and `credentials`
  protos under `/api/2.1/unity-catalog/…`, gated by a provider token.
  Recipient creation issues a token and an activation URL, and
  `GET /api/1.0/opensharing/activations/{code}` returns the profile.

`olai-sharing-neon` hosts the same router as a Neon Function. It mirrors
Headwaters (`headwaters-neon`): a napi `cdylib`, a thin dependency-free
TypeScript fetch handler, and a `release-neon` profile. Configuration comes from
the environment (`SHARING_SERVER_CONFIG_YAML`, plus Neon's `DATABASE_URL` /
`DATABASE_URL_UNPOOLED`). Migrations run out of band (`just sharing-neon
migrate`).

Size is the binding constraint: Neon allows 10 MiB zipped and 64 MiB unzipped.
All builds below are for `aarch64-unknown-linux-gnu.2.28`, with fat LTO, one
codegen unit and stripped symbols. Measured on 2026-10-29 with the standalone
server binary, which links the same dependency graph as the addon (kernel,
arrow/parquet, `object_store` with AWS/Azure/GCS, sqlx, rustls):

| `opt-level` | `panic` | unzipped | zipped |
|---|---|---|---|
| 3 | unwind | 27.9 MiB | 11.1 MiB (over the limit) |
| `"z"` | abort | 15.3 MiB | 6.7 MiB |
| `"z"` (`release-neon`, the actual addon bundle) | unwind | 17.5 MiB | 7.8 MiB |

The bundle loads and passes the smoke suite on `node:24-bookworm-slim`
(linux-arm64) via `just sharing-neon test-arm64`. It has not been deployed to a
Neon branch yet.

`release-neon` therefore uses `opt-level = "z"`. It keeps `panic = "unwind"`,
so a panic fails the request instead of the isolate. `just sharing-neon build`
gates the real bundle on both limits. If it gets tight, the next levers are:

1. `panic = "abort"`;
2. per-cloud `object_store` features;
3. trimmed parquet codecs.

## Delivery

One GitHub stack, one PR per layer:

1. This document.
2. Extract `olai-uc-credentials` (vending, storage locations, presign) out of
   `uc-server`, with no behavior change.
3. `sharing-api` ports split and kernel Delta format. DataFusion is dropped,
   pagination and capability parsing are fixed.
4. Storage access on the recipient surface: presigned `/query`, credential
   endpoints, the dual-prefix router and bearer auth.
5. `olai-sharing-server`: Postgres store, sealed secrets with lazy rewrap,
   UC REST catalog source, delegated and local access, admin API, binary,
   end-to-end tests.
6. The Neon Function host (`olai-sharing-neon`), with the size measurement
   above.

## Deferred

Tracked as follow-up issues:

- Change data feed.
- `responseformat=delta`.
- Async query.
- `partitions` filtering.
- Iceberg REST and UC-Delta catalog sources.
- An Iceberg REST view of shared tables.
- OIDC recipients.
- The upstream-shaped `/provider/*` admin API.
- A cloud KMS `KeyProvider`.
