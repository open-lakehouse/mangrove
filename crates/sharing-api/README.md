# olai-uc-sharing-api

Portable Unity Catalog **Open Sharing** / **Delta Sharing** REST API.

This crate owns the sharing *semantics* — share/schema/table discovery, the
kernel-backed NDJSON query path (table version / metadata / query), and the
storage-backed asset surface (volumes, agent skills) — behind a narrow backend
[port](crate::backend::SharingBackend). Any server can serve the identical
`/api/v1/delta-sharing` and `/api/v1/open-sharing` surfaces by implementing
`SharingBackend` over its own share store, table/volume resolution, credential
vending, and authorization; all the sharing business logic is shared here, so the
behavior is identical by construction.

Unlike the fork-free [`olai-uc-delta-api`](https://crates.io/crates/olai-uc-delta-api),
this crate deliberately owns the DataFusion + `delta-kernel` query path (via
[`olai-uc-datafusion`](https://crates.io/crates/olai-uc-datafusion)'s
`ReconciledLogProvider`), isolating those git-pinned dependencies behind one
crate so downstream servers stay free of them.

The wire types come from
[`olai-uc-sharing-client`](https://crates.io/crates/olai-uc-sharing-client).

## Status

The Delta Sharing tabular surface and the Open Sharing asset surface are
implemented. Recent protocol additions — Change Data Feed (`/changes`),
asynchronous queries (`POST /queries/{id}`), and `responseformat=delta` serving —
have their request/response types defined but return `501 Not Implemented`.
