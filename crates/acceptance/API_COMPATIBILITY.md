# Unity Catalog API Compatibility Report

This document records how `unitycatalog-rs`'s API definitions compare against the two
reference Unity Catalog implementations we target for client compatibility:

- **OSS Unity Catalog (Java)** — OpenAPI `api/all.yaml` at tag
  [`v0.4.1`](https://github.com/unitycatalog/unitycatalog/blob/v0.4.1/api/all.yaml),
  the version of the `newfrontdocker/unitycatalog:v0.4.1` image we run in CI
  (`dev/uc-oss.compose.yaml`, `integration_oss_java` job).
- **Managed Databricks** — the `/api/2.1/unity-catalog` REST surface
  ([Catalogs](https://docs.databricks.com/api/workspace/catalogs),
  [Schemas](https://docs.databricks.com/api/workspace/schemas),
  [Tables](https://docs.databricks.com/api/workspace/tables),
  [Grants](https://docs.databricks.com/api/workspace/grants)).

"Ours" = the proto definitions under `proto/unitycatalog/unitycatalog/*/v1/svc.proto`,
the generated models in `crates/common/src/models/gen/`, the server routes in
`crates/server/src/rest/routers/mod.rs`, and the client in `crates/client/`.

The goal is clients that work against **managed Databricks, OSS Java, and our own Rust
server**. This report is scoped to the **resources OSS Unity Catalog actually
implements** (plus Delta Sharing resources that only exist on our server + Databricks).

---

## Resource & operation matrix

Legend: ✅ defined · ❌ absent · — n/a

| Resource | Ours | OSS Java v0.4.1 | Managed DBX | Notes |
|---|---|---|---|---|
| **Catalogs** C/G/L/U/D | ✅ | ✅ | ✅ | aligned |
| **Schemas** C/G/L/U/D | ✅ | ✅ | ✅ | aligned |
| **Tables** C/G/L/D (no update) | ✅ | ✅ (no PATCH) | ✅ (no PATCH) | aligned — table update intentionally absent in all three |
| Tables: list summaries / exists | ✅ | partial | ✅ | OSS Java has no `/table-summaries`; we keep them, tagged DBX/Rust-only |
| **Volumes** C/G/L/U/D | ✅ | ✅ | ✅ | enum wire-format fixed — see below |
| **Functions** C/G/L/D (+update) | ✅ | ✅ (no PATCH) | ✅ | OSS Java has no function update; ours/DBX do |
| **External Locations** C/G/L/U/D | ✅ | ✅ | ✅ | aligned |
| **Credentials** (storage) C/G/L/U/D | ✅ | ✅ | ✅ | aligned |
| **Temporary Credentials** (table/path/volume/model-version) | ✅ | ✅ | ✅ | aligned |
| **Shares** + share permissions | ✅ | ❌ (no sharing server) | ✅ | OSS-Rust / DBX only |
| **Recipients** C/G/L/U/D | ✅ | ❌ | ✅ | OSS-Rust / DBX only |
| **Providers** C/G/L/U/D | ✅ | ❌ | ✅ | OSS-Rust / DBX only |
| **Grants / Permissions** (`/permissions/{securable_type}/{full_name}`) | ❌ (share-scoped only) | ✅ | ✅ (`/grants/...` + `geteffective`) | **deferred** — see below |
| **Metastore summary** (`/metastore_summary`) | ❌ | ✅ | ✅ | minor gap, deferred |
| **Registered Models / Versions** | ✅ | ✅ | ✅ | aligned (incl. `/temporary-model-version-credentials`) |
| **Delta commits** (`/delta/preview/commits`) | ❌ | ✅ | ✅ | out of scope (commit coordinator) |

### Conventions — aligned
- **Pagination**: `max_results` + `page_token` query params, `next_page_token` in
  responses, matching OSS and Databricks (`...catalogs/v1/svc.proto:18-36`).
- **Path shapes / field names**: `preserve_proto_field_names=true` keeps snake_case JSON
  field names matching the REST APIs.

---

## Wire-format fix: enum string values (fixed in this change)

The generated serde (`neoeinstein-prost-serde:v0.3.1`, configured in `buf.gen.yaml`)
derives each enum's JSON string **directly from the proto enum value name** — there is no
prefix-stripping option and no `json_name` override. Most of our enums already follow the
correct convention: the type prefix appears **only** on the zero/`_UNSPECIFIED` value, and
real values are bare (`TableType.MANAGED`, `DataSourceFormat.DELTA`, `CatalogType`,
`RoutineBody`, `Purpose`, `recipients.AuthenticationType`, …).

Two enums were inconsistent outliers that carried the prefix on real values and therefore
serialized to strings neither OSS nor Databricks accept:

| Enum | Was (wire) | Now (wire) | Expected by OSS/DBX |
|---|---|---|---|
| `VolumeType` (`proto/.../volumes/v1/models.proto`) | `VOLUME_TYPE_MANAGED` / `VOLUME_TYPE_EXTERNAL` | `MANAGED` / `EXTERNAL` | `MANAGED` / `EXTERNAL` |
| `ProviderAuthenticationType` (`proto/.../providers/v1/models.proto`) | `PROVIDER_AUTHENTICATION_TYPE_TOKEN` / `..._OAUTH_CLIENT_CREDENTIALS` | `TOKEN` / `OAUTH_CLIENT_CREDENTIALS` | `TOKEN` / `OAUTH_CLIENT_CREDENTIALS` |

The `VolumeType` mismatch was the documented OSS Java failure (the
`VOLUME_TYPE_MANAGED` 500 in `JOURNEY_CATALOG.md`). `ProviderAuthenticationType` was also
inconsistent with its sibling `recipients.AuthenticationType`, which already used the bare
form.

**Fix**: renamed the prefixed values to bare names in the two `.proto` files (keeping
`*_UNSPECIFIED = 0`) and regenerated via `just generate-proto` / `buf generate`. The Rust
variant identifiers (`VolumeType::Managed`, `ProviderAuthenticationType::Token`) are
unchanged — prost already stripped the prefix — so there is no call-site churn; only the
serialized strings change. Evidence after regeneration:
`crates/common/src/models/gen/unitycatalog.volumes.v1.serde.rs` now emits `"MANAGED"` /
`"EXTERNAL"`, and `...providers.v1.serde.rs` emits `"TOKEN"` /
`"OAUTH_CLIENT_CREDENTIALS"`.

---

## Gaps & deferrals

| Item | Status | Rationale / tracking |
|---|---|---|
| **Grants / Permissions** general service | **Deferred** | We only have share-scoped permissions today. The general grants service (OSS `/permissions/{securable_type}/{full_name}`; Databricks `/grants/...` + `geteffective`) is a later task that requires integrating a **policy engine** — grant enforcement is meaningless without it, so it is sequenced after that integration. Tracked by [#29 "Support Grants API"](https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/29) (relates to [#28 OpenFGA-backed authorization policy](https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/28)). |
| **Metastore summary** (`/metastore_summary`) | **Deferred** | Minor read-only endpoint; low priority. Tracked by [#149](https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/149). |
| **Delta commits** (`/delta/preview/commits`) | Out of scope | Belongs to the commit-coordinator surface, not the catalog API. |

The enum wire-format fix in this change closes
[#141](https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/141)
("client sends protobuf-style enum values (`VOLUME_TYPE_MANAGED`) that UC OSS Java
rejects"). The new `temporary_volume_credentials` journey complements
[#119](https://github.com/unitycatalog-incubator/unitycatalog-rs/issues/119) (server-side
handler for `/temporary-volume-credentials`).

---

## Integration-test coverage

CRUD coverage is exercised by the acceptance **journeys** (`crates/acceptance/`, see
`JOURNEY_CATALOG.md`). The client already exposes every operation listed as ✅ above, so
remaining gaps are journey-coverage gaps, not client gaps. Coverage after this change:

| Resource | C | G | L | U | D | Journey |
|---|---|---|---|---|---|---|
| Catalogs | ✅ | ✅ | ✅ | ✅ | ✅ | `enhanced_catalog` (C/G/L/D); catalog update covered in `schema_lifecycle` |
| Schemas | ✅ | ✅ | ✅ | ✅ | ✅ | `schema_lifecycle` |
| Tables | ✅ | ✅ | ✅ | — | ✅ | `table_managed_lifecycle` (+ summaries/exists) |
| Volumes | ✅ | ✅ | ✅ | ✅ | ✅ | `volume_managed_lifecycle` |
| Functions | ✅ | ✅ | ✅ | ✅ | ✅ | `function_lifecycle` |
| External Locations | ✅ | ✅ | ✅ | ✅ | ✅ | `external_location_lifecycle` |
| Credentials | ✅ | ✅ | ✅ | ✅ | ✅ | `credential_lifecycle` |
| Temporary Credentials | — | — | — | — | — | `temporary_{table,path,volume}_credentials` |
| Shares | ✅ | ✅ | ✅ | ✅ | ✅ | `share_lifecycle` |
| Recipients | ✅ | ✅ | ✅ | ✅ | ✅ | `recipient_lifecycle` |
| Providers | ✅ | ✅ | ✅ | ✅ | ✅ | `provider_lifecycle` |

Journeys touching Databricks-only or external-storage-dependent resources are tagged via
`JourneyMetadata.implementations` so the `oss_java` CI run only executes compatible ones.
Fixtures are recorded against managed Databricks (`UC_INTEGRATION_RECORD=true`); see
`JOURNEY_CATALOG.md` for the exact command.
