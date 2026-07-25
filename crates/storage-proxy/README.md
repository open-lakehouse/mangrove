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

## Storage-access posture: `proxy` vs `direct`

A UC server announces its posture at the server-root `/capabilities` endpoint:

```jsonc
// proxy enabled
{ "storageAccess": "proxy",
  "storageProxy": { "basePath": "/storage-proxy", "conditionalWrites": true } }
// proxy disabled (the default)
{ "storageAccess": "direct" }
```

**`proxy`** — the browser/wasm engine routes every object byte through this
crate's `/storage-proxy/{securable}/{key}` surface. The wasm store is a stock
`object_store::http::HttpStore` pointed at the same origin, so it never issues a
cross-origin request and never speaks a cloud-native protocol. This is the
recommended default for a fresh deployment: it works for S3, GCS, and Azure
alike, and requires **no CORS configuration on the storage account**.

**`direct`** — the browser vends a credential and reads/writes cloud object
storage directly (the historical behavior; kept as an opt-in fast path). It
avoids the extra server hop and its egress cost, but requires **per-cloud CORS
configuration on the storage host** (see below) and, on wasm, is Azure-only —
the wasm build gates out the S3/GCS `object_store` backends, and SigV4/GCS
signing cannot run in the browser regardless.

The wasm client (`olai-uc-object-store`) discovers the posture from
`/capabilities` once when its `UnityObjectStoreFactory` is built and routes
accordingly — no per-call flag or JS wiring. A server that predates the endpoint,
or one that returns anything other than `"proxy"`, is treated as `direct`.

### Why `direct` needs CORS (and `proxy` does not)

A browser `fetch()` to a different origin than the page is subject to CORS: the
**storage host** must return `Access-Control-Allow-Origin` (and, for ranged
reads, allow the `Range` request header and expose `Content-Range` / `ETag`)
or the browser refuses to let the page read the response. A presigned S3 URL or
an Azure SAS URL solves *authentication* but **not** CORS — the request is still
cross-origin. So `direct` obliges every customer to configure a CORS policy on
each bucket/container/account (identical burden on AWS, GCS, and Azure).

`proxy` sidesteps this entirely: the browser only ever talks to its own origin,
so there is no cross-origin request and nothing to configure on the storage host.
All cloud signing happens server-side, where the native AWS/Azure/GCP code already
works.

### Egress and the CDN escape valve

Under `proxy`, pruned object bytes transit the UC server (in-browser projection,
row-group pruning, and Delta data-skipping run first, so only the needed byte
ranges move). For high-egress, wide, or very popular previews this server
bandwidth can dominate. When it does, front the proxy with a CDN or object-store
edge cache, or fall back to `direct` (accepting the per-cloud CORS setup) for the
hot paths. Measure bytes-through-server before reaching for either.

### Authentication note

When `proxy` is enabled in a shared / multi-tenant deployment, pair it with a
real request identity so vends are attributed to the caller rather than the
over-privileged anonymous principal. The server ships a `ReverseProxyAuthenticator`
(config `auth.mode: reverse-proxy`) that trusts a forwarded-user header set by an
upstream proxy; see the server crate's `config` module.
