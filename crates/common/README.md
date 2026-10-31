# olai-uc-common

**Shared Unity Catalog types and utilities used by both the server and client
crates.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_common` (imports and paths use that
> name); the crate is published to crates.io as `olai-uc-common` while the naming
> settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

Most of this crate is the Unity Catalog data model, generated from the protobuf
definitions in `proto/` and re-exported from the `models` module. On top of that
it collects the hand-written pieces both sides of the API depend on: the
crate-wide error type, the `uc://` reference scheme, storage-abstraction traits,
envelope encryption for secrets, and the metric-view definition parser.

Because it serves such different consumers — a full REST server, a thin client,
Python and Node bindings — it is feature-flag heavy. Most functionality is gated
so that downstream crates pull in only what they need.

## Quick start

Add the dependency (the `package` rename keeps the `unitycatalog_common` import
path):

```toml
[dependencies]
unitycatalog-common = { package = "olai-uc-common", version = "0.0.1" }
```

## Feature flags

`rest-client` and `all-resources` are on by default. The rest gate optional integrations:

| Feature               | Default | What it enables                                                                                 |
| --------------------- | :-----: | ----------------------------------------------------------------------------------------------- |
| `all-resources`       |   yes   | Every generated model package (see [Resource features](#resource-features)).                    |
| `rest-client`         |   yes   | Client implementation for the Unity Catalog REST APIs.                                          |
| `grpc`                |         | Generated tonic gRPC service code (`tonic` + `tonic-prost`).                                    |
| `metric-view`         |         | Metric-view YAML parsing and SQL dependency extraction.                                         |
| `axum`                |         | `FromRequest` / `FromRequestParts` impls for request types.                                     |
| `store`               |         | Storage-abstraction trait (`ResourceStore`) and services. Implies `all-resources`.              |
| `sqlx`                |         | `sqlx` trait derives on selected types for backends.                                            |
| `python`              |         | `pyclass` derives on generated messages (used by the Python bindings). Implies `all-resources`. |
| `node`                |         | `#[napi]` derives on generated enums (used by the Node bindings).                               |
| `storage-credentials` |         | Conversions to the vended `olai-storage-credentials` model (wasm-compatible).                   |
| `credential-vending`  |         | Conversions to the registered `olai-credential-vending` model (native only).                    |
| `integration`         |         | Test helpers for custom handler / router implementations.                                       |

### Resource features

Each generated model package (`unitycatalog.<resource>.<version>`) sits behind a
feature named after its module, so a consumer that needs only a few resources can
skip compiling the rest (about 60% less `common` build time for a five-resource
subset):

`agent_skills`, `agents`, `catalogs`, `credentials`, `external_locations`,
`functions` (implies `tables`), `model_versions`, `policies`, `registered_models`,
`schemas`, `staging_tables`, `tables`, `tags`, `temporary_credentials`, `volumes`.

```toml
unitycatalog-common = { package = "olai-uc-common", version = "…", default-features = false, features = [
  "rest-client", "tables", "volumes",
] }
```

Resource features unify across the dependency graph: any crate in your build that
depends on `olai-uc-common` with default features turns every resource back on.
The `Resource` enum and the `ObjectLabel` vocabulary span every resource and
need `all-resources`. The axum extractors follow the resource features.

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
