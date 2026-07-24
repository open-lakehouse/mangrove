# olai-uc-storage-proxy

A same-origin **storage byte-proxy** for Unity Catalog. A browser/wasm query
engine points a stock `object_store::http::HttpStore` at this proxy instead of
talking to cloud object storage directly; the proxy authorizes the request,
vends a scoped credential, and **streams object bytes** (read + write) to/from
cloud storage server-side.

Because the wasm side never speaks a cloud-native protocol (SigV4, Azure SAS,
GCS auth), this makes S3/GCP work on wasm for free and removes the per-cloud CORS
configuration burden — the browser only ever talks to its own origin.

The crate owns the proxy *semantics* — the wire contract (ranged GET / HEAD /
whole-object PUT with server-enforced `If-Match`), the streaming HTTP mapping,
and the confused-deputy path-scope guard — behind a narrow
[`StorageProxyBackend`] port. Any server serves the identical surface by
implementing the port over its own storage, credential vending, and
authorization.

## Layout

- `error` — the `ProxyError` contract + its `IntoResponse`.
- `backend` — the `StorageProxyBackend` port + `ProxyReq` / `Securable` /
  `ProxyVerb` request vocabulary.
- `handler` — the streaming `StorageProxyHandler` blanket impl over the port.
- `router` — the state-agnostic, host-composable axum router.
- `config` — the `storageAccess` capability helper.
- `client` (feature `client-arm`) — a portable backend backed by a
  `UnityObjectStoreFactory`, working against any UC server given `{baseUrl, token}`.
- `testing` (feature `testing`) — an in-memory backend for the wire-contract tests.

This crate is part of the [mangrove](https://github.com/open-lakehouse/mangrove)
Unity Catalog implementation.
