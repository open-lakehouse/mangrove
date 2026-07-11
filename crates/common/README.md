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

`rest-client` is on by default. The rest gate optional integrations:

| Feature       | Default | What it enables                                                              |
| ------------- | :-----: | ---------------------------------------------------------------------------- |
| `rest-client` |   yes   | Client implementation for the Unity Catalog REST APIs.                       |
| `grpc`        |         | Generated tonic gRPC service code (`tonic` + `tonic-prost`).                 |
| `metric-view` |         | Metric-view YAML parsing and SQL dependency extraction.                      |
| `axum`        |         | `FromRequest` / `FromRequestParts` impls for request types.                  |
| `store`       |         | Storage-abstraction traits (`ResourceStore`, `SecretManager`) and services.  |
| `sqlx`        |         | `sqlx` trait derives on selected types for backends.                         |
| `python`      |         | `pyclass` derives on generated messages (used by the Python bindings).       |
| `node`        |         | `#[napi]` derives on generated enums (used by the Node bindings).            |
| `integration` |         | Test helpers for custom handler / router implementations.                    |

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
