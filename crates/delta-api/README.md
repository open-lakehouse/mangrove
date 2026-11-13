# olai-uc-delta-api

**Portable Unity Catalog Delta v1 REST API — the wire models, the managed-table
contract, and a backend-agnostic port, all in one dependency.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_delta_api` (imports and paths use
> that name); the crate is published to crates.io as `olai-uc-delta-api` while
> the naming settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

This crate owns the *semantics* of the Unity Catalog Delta v1 REST API — the
hand-written wire models, the catalog-managed table contract, the commit
coordinator, and the `updateTable` action dispatcher — behind a single narrow
trait. Any server can serve the identical `/delta/v1` surface by implementing
that trait over its own storage, credential vending, and authorization. All the
Delta business logic lives here, so two servers built on it behave identically.

**You don't need to be a Unity Catalog server to use it.** The Delta v1 API is a
self-contained surface, and so is this crate. Any data catalog can adopt the
Delta v1 endpoints on their own — exposing Delta tables to Delta clients —
without implementing the rest of the UC API. If you can resolve a table to a
storage location and vend a credential for it, you can serve Delta v1.

It has no dependency on any UC server crate. Besides the usual `serde`,
`thiserror`, `uuid`, `tracing` and `async-trait`, it pulls `axum` (behind the
default `server` feature) and trestle's `olai-storage-credentials`, whose
credential model and Delta wire shape the credential endpoints use. With
`default-features = false` only the wire models remain, which build for wasm.

## When you'd reach for this

- You run **any data catalog** and want to **add Delta v1 support** so Delta
  clients can read and write your tables — without adopting the rest of the
  Unity Catalog API. The port asks only for what a catalog already does.
- You are **building a Unity Catalog–compatible server** and want a spec-faithful
  Delta v1 endpoint without re-deriving the managed-table contract, the
  protocol-version negotiation, or the commit arbitration rules.
- You need the **wire types** as plain serde DTOs to build a client or a proxy.
- You want an **in-memory reference backend** to test against known-good Delta
  semantics (behind the `testing` feature).

## Architecture at a glance

```
        HTTP  /delta/v1/...
           │
   ┌───────▼────────┐   get_router::<T, Cx>()   axum::Router — 12 operations
   │     router     │
   └───────┬────────┘
   ┌───────▼────────┐   DeltaApiHandler          all Delta business logic:
   │    handler     │   (blanket impl)           contract, updateTable dispatch,
   └───────┬────────┘                            loadTable list, commit arbitration
   ┌───────▼────────┐   DeltaBackend  ◄────────  YOU implement this
   │  your backend  │   (the port)               storage · credentials · authz
   └────────────────┘
```

You implement [`DeltaBackend`]; you get [`DeltaApiHandler`] (via a blanket impl)
and the axum [`get_router`] for free.

## Quick start

Add the dependency (the rename keeps the `unitycatalog_delta_api` import path):

```sh
cargo add olai-uc-delta-api --rename unitycatalog-delta-api
```

While the crate is at `0.0.x`, every release may be breaking, and Cargo treats a
`0.0.x` requirement as exact. Expect to bump the version by hand, and read the
changelog when you do.

Implement the port, then mount the router:

```rust
use unitycatalog_delta_api::{DeltaBackend, get_router};

// 1. Implement `DeltaBackend<Cx>` for your server state, where `Cx` is your
//    per-request context (auth principal, request metadata, …). You provide
//    ~15 data-access methods — resolve_table, create_table_row, vend_*, an
//    authorize hook, and a commit coordinator — over your own storage. No
//    Delta semantics required; the handler supplies those.

// 2. Mount the router. `get_router` reads `Cx` from the request extensions
//    (your auth middleware inserts it); a request without one is a 401. Use
//    `router_with_context` to build `Cx` with your own async closure, or to
//    nest the routes into a router that carries its own axum state.
let app: axum::Router = get_router::<MyBackend, MyContext>(my_backend);
```

The router matches `openapi/delta.yaml` and mounts all twelve operations:
`getConfig`, `createStagingTable`, `createTable`, `loadTable`, `updateTable`,
`deleteTable`, `tableExists`, `renameTable`, table/staging/path credential
vending, and `reportMetrics`.

## What the port gives you, and what it asks for

**You get, unconditionally:**

- The **managed-table contract** — required protocol features, fixed table
  properties, and the Delta ↔ UC column mapping — enforced on create/update.
- The **`updateTable` action dispatcher**, applying actions in the reference's
  canonical order.
- **`loadTable` commit-list construction** and the **credential → config**
  mapping.
- **Commit arbitration and backfill** via the shared commit coordinator.
- **`getConfig`** capability negotiation: the advertised endpoint list is driven
  by your [`DeltaCapabilities`], and the served protocol version is negotiated
  against the client's `protocol-versions`.
- A **decoupled error contract**: you return [`DeltaBackendError`] variants; the
  handler maps them to the correct HTTP status and Delta error body.

**You provide** — pure data access, no Delta logic:

- Table resolution / create / update / delete / rename over your store.
- Credential vending for tables, staging tables, and temporary paths.
- Staging-table reservation.
- A single **`authorize`** hook, called with a [`DeltaAction`] before every
  user-facing operation. The data methods must *not* re-authorize — the handler
  has already authorized the operation.

## Contracts worth reading before you implement the port

- **Authorization comes before resolution.** Table actions name the table by
  its three-part name, and `authorize` runs before `resolve_table`, so a caller
  without access cannot tell a missing table from a forbidden one. If your
  permissions are keyed by table id, resolve the name inside `authorize` and
  deny (not 404) a table that doesn't exist. Staging credentials are additionally
  bound to the reservation's creator through `DeltaAction::AdoptStaging`.
- **`apply_update` is atomic.** The spec says the actions of one `updateTable`
  apply atomically, and a 409 tells the client its commit did not land. A
  transactional backend should override `DeltaBackend::apply_update` and run
  the etag compare-and-swap, the metadata write and the commit ratification in
  one transaction. The default writes metadata first, then ratifies, and undoes
  the metadata write when the commit is rejected. It never leaves a ratified
  commit behind a 409, but readers can briefly see metadata ahead of the log.
- **Retries are deduplicated by commit file name.** A retried `add-commit` that
  already landed returns 200 with the current table. A different file at a taken
  version is a 409. A retry older than the coordinator's retained rows is a 500
  `CommitStateUnknownException`. Persistent coordinators must keep backfilled
  rows for the retention window, and should implement `find_commit`, for this to
  work.
- **Port types are `#[non_exhaustive]`.** Build `ResolvedTable`,
  `StagingReservation`, `VendedCredential` and `DeltaCapabilities` with their
  constructors, then assign the public fields. Match `DeltaBackendError`,
  `CommitError` and `DeltaAction` with a wildcard arm. The wire enums
  (`DeltaTableType`, `DeltaDataSourceFormat`, `DeltaErrorType`) gain an
  `Unknown` variant, so a client keeps parsing responses from a newer server.
- **5xx errors are redacted.** For `Internal`, `Backend` and the commit-state
  errors the response carries a generic message and a reference id. The detail,
  with its source chain, is logged at `error` level under that id. 4xx messages
  are returned as written, so keep secrets out of them.

## Testing feature

Enable the `testing` feature to get `InMemoryDeltaBackend`, an in-memory
implementation of the port:

```toml
[dev-dependencies]
unitycatalog-delta-api = { package = "olai-uc-delta-api", version = "<current>", features = ["testing"] }
```

Use it to exercise the Delta semantics without a real backend, or as a
known-good oracle while wiring up your own port implementation. It is
deliberately permissive on authorization (apart from the staging creator check)
and vends a fixed fake credential. The point is to exercise the contract,
dispatcher, and commit arbitration, not access control. Its `apply_update` is
atomic, so it is also the reference for that contract.

## Module map

| Module        | What's in it                                                    |
| ------------- | --------------------------------------------------------------- |
| `models`      | Hand-written serde wire DTOs (kebab-case JSON)                  |
| `backend`     | The `DeltaBackend` port + its coordinate / request / spec types |
| `handler`     | The `DeltaApiHandler` trait + the generic blanket impl          |
| `router`      | The axum router mounting all twelve operations                  |
| `contract`    | Managed-table contract + Delta ↔ UC column mapping              |
| `coordinator` | The commit coordinator (arbitration + backfill)                 |
| `config`      | `getConfig` capability list + protocol-version negotiation      |
| `authz`       | The `DeltaAction` vocabulary the handler authorizes with        |
| `column`      | The portable UC column model used by the contract               |
| `error`       | The `DeltaApiError` / `DeltaBackendError` contract              |
| `testing`     | `InMemoryDeltaBackend` (feature `testing`)                      |

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
