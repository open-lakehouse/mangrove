# olai-uc-client

**Async Rust client for the Unity Catalog REST API.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_client` (imports and paths use that
> name); the crate is published to crates.io as `olai-uc-client` while the naming
> settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

`UnityCatalogClient` is the entry point. Construct it from a base URL and an auth
token, then reach a resource through the accessor for that resource. Each accessor
returns a scoped sub-client or a request builder; list builders implement
`IntoFuture` (so you can `.await` them directly) and `into_stream()` for
auto-paginated iteration.

## Quick start

Add the dependency (the `package` rename keeps the `unitycatalog_client` import
path):

```toml
[dependencies]
unitycatalog-client = { package = "olai-uc-client", version = "0.0.1" }
```

Construct the client and call a resource:

```rust
use unitycatalog_client::UnityCatalogClient;
use futures::TryStreamExt;
use url::Url;

let base = Url::parse("https://example.com/api/2.1/unity-catalog/")?;
let client = UnityCatalogClient::new_with_token(base, "dapi...");

// Fetch one catalog by name.
let catalog = client.catalog("main").get().await?;

// List tables in a schema, streaming across pages.
let tables: Vec<_> = client
    .list_tables("main", "default")
    .into_stream()
    .try_collect()
    .await?;
```

Fallible calls return `Result<T, Error>`. `Error` distinguishes UC API errors
(`UcApiError`) from Delta error envelopes and offers predicates such as
`is_not_found()`, `is_already_exists()`, and `is_commit_conflict()` for control
flow.

## The client surface

Most resource accessors on `UnityCatalogClient` are generated from the API spec.
Two surfaces are hand-written and have their own accessors:
`temporary_credentials()` (credential vending with name → UUID resolution) and
`delta_v1()` (the `/delta/v1` Delta REST API).

| Resource               | Accessor                                | Scoped client               |
| ---------------------- | --------------------------------------- | --------------------------- |
| Catalogs               | `catalog(name)` / `list_catalogs()`     | `CatalogClient`             |
| Schemas                | `schema(...)` / `list_schemas(...)`     | `SchemaClient`              |
| Tables                 | `table(...)` / `list_tables(...)`       | `TableClient`               |
| Volumes                | `volume(...)` / `list_volumes(...)`     | `VolumeClient`              |
| Functions              | `function(...)` / `list_functions(...)` | `FunctionClient`            |
| Credentials            | `credential(...)` / `list_credentials()`| `CredentialClient`          |
| External locations     | `external_location(...)`                | `ExternalLocationClient`    |
| Shares                 | `share(...)` / `list_shares()`          | `ShareClient`               |
| Providers              | `provider(...)` / `list_providers()`    | `ProviderClient`            |
| Recipients             | `recipient(...)` / `list_recipients()`  | `RecipientClient`           |
| Policies / tag policies| `list_policies(...)`                    | `PolicyClient` / `TagPolicyClient` |
| Agents / agent skills  | `agent(...)` / `agent_skill(...)`       | `AgentClient` / `AgentSkillClient` |
| Staging tables         | `staging_tables_client()`               | `StagingTableClient`        |
| Temporary credentials  | `temporary_credentials()`               | `TemporaryCredentialClient` |
| Delta v1 API           | `delta_v1()`                            | `DeltaV1Client`             |

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
</content>
</invoke>
