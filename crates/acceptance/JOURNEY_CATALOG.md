# API-coverage catalog

The conformance battery's coverage of the Unity Catalog API, organized by
securable. Each check drives one securable's lifecycle against a live server; see
[`README.md`](./README.md) for how the batteries and quarantine work.

## Batteries

- **baseline** (`baseline_checks()`) — portable to any UC implementation,
  including UC OSS Java v0.5.0. This is the set the `integration_oss_java` CI job
  runs.
- **extended** (`extended_checks()`) — baseline plus everything only our Rust
  server (and mostly managed Databricks) implements. This is what
  `conformance_oss_rust` and `conformance_managed_databricks` run.

## Coverage inventory

| Check | Securable(s) | Battery | Notes |
|---|---|---|---|
| `catalog_crud` | Catalogs | baseline | create/list/get/update |
| `catalog_hierarchy` | Catalogs, Schemas | baseline | catalog + N schemas |
| `schema_lifecycle` | Schemas | baseline | create/get/list/update |
| `managed_table_lifecycle` | Tables | baseline | +summaries +exists |
| `metric_view_lifecycle` | Tables (`METRIC_VIEW`) | baseline | needs UC OSS ≥ v0.5.0 |
| `managed_volume_lifecycle` | Volumes | baseline | |
| `function_lifecycle` | Functions | baseline | create/get/list |
| `function_update` | Functions | baseline | UC OSS has no function update |
| `credential_lifecycle` | Credentials | extended | self-skips without cloud identity |
| `external_location_lifecycle` | ExternalLocations | extended | self-skips without cloud storage |
| `external_table_lifecycle` | Tables (external) | extended | self-skips without cloud storage |
| `external_volume_lifecycle` | Volumes (external) | extended | self-skips without cloud storage |
| `temporary_table_credentials` | TemporaryCredentials | extended | |
| `temporary_path_credentials` | TemporaryCredentials | extended | self-skips without cloud storage |
| `temporary_volume_credentials` | TemporaryCredentials | extended | |
| `share_lifecycle` | Shares | extended | our-server / DBX only |
| `recipient_lifecycle` | Recipients | extended | our-server / DBX only |
| `provider_lifecycle` | Providers | extended | our-server / DBX only |
| `policy_lifecycle` | Policies (ABAC) | extended | mangrove-only |
| `tag_policy_lifecycle` | TagPolicies | extended | mangrove-only |
| `entity_tag_assignment_lifecycle` | EntityTagAssignments | extended | mangrove-only |
| `agent_lifecycle` | Agents | extended | mangrove-only |
| `agent_skill_lifecycle` | AgentSkills | extended | mangrove-only |
| `lakehouse_hierarchy` | Catalogs/Schemas/Tables/Volumes | extended | cross-resource |
| `governance_setup` | Credentials/ExternalLocations/Tables | extended | self-skips without cloud storage |

## Known-failing worklist (our Rust server)

These surfaces are *attempted* by the battery but currently fail against our
`uc-server`; they are quarantined in `conformance::known_failing` so CI stays
green, and each is a follow-up to fix. Replace the `#TODO` issue refs when the
tickets are filed, and delete the quarantine entry once the surface passes (the
run flags it as an unexpected pass if left behind).

| Check | Symptom | Root cause |
|---|---|---|
| `managed_table_lifecycle` | 400 "managed tables require storage_location to be the staging location" | Managed tables must go through the `/delta/v1` staging flow, not a bare `create_table`. |
| `share_lifecycle` | same as above | Creates a managed table as its fixture. |
| `lakehouse_hierarchy` | same as above | Creates managed tables. |
| `temporary_table_credentials` | same as above | Needs a managed table fixture. |
| `temporary_volume_credentials` | 404 | `temporary-volume-credentials` not served. |
| `tag_policy_lifecycle` | 405 | `tag-policies` create not served. |
| `entity_tag_assignment_lifecycle` | 404 | `entity-tag-assignments` not served. |
| `policy_lifecycle` | 400 "row filter policies require row_filter.function_name" | A valid ABAC policy needs a backing function; the check does not yet create/wire one. |

## Non-goals

Resources UC OSS implements but our server does not, and which the battery
deliberately does **not** cover:

- **RegisteredModels / ModelVersions** — not implemented (deferred, #148).
- **General Grants/Permissions** (`/permissions/{securable_type}/{full_name}`) —
  only share-scoped permissions exist today (deferred, #29).
- **Metastore summary** (`/metastore_summary`) — not implemented (deferred, #149).
- **Users / control-plane API** — not implemented.

## Deferred (Phase 2)

- **`/delta/v1` (CCv2 commit API)** — a `checks/delta.rs` exercising
  `client::delta_v1` against a live server, to be added to `extended_checks()`.
  Fixing the managed-table staging flow above is a prerequisite for the
  managed-table checks and overlaps this work.
