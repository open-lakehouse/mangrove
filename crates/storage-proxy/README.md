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
- `auth` / `cli` (feature `bin`) — the request-auth layer and the CLI that back
  the standalone `storage-proxy` binary (see below).

## Running standalone

Besides being embedded in a UC server (mounted at `/storage-proxy`), the proxy
runs as its own deployable — a container that sits at the browser's origin and
forwards to an upstream Unity Catalog for credential vending. It has no database,
so it needs no migrations; the CLI is just `serve` and `healthcheck`. Build the
binary with the `bin` feature:

```bash
cargo run -p olai-uc-storage-proxy --features bin --bin storage-proxy -- \
  serve --upstream-url http://uc:8080/api/2.1/unity-catalog/ --port 8080
```

or, as a container:

```bash
docker build -f crates/storage-proxy/Dockerfile -t storage-proxy .
docker run --rm -p 8080:8080 ghcr.io/open-lakehouse/storage-proxy:<version> \
  serve --upstream-url http://uc:8080/api/2.1/unity-catalog/
```

The image's `ENTRYPOINT` is the binary and its default `CMD` is `serve`, so a
bare `docker run` starts the proxy; pass `healthcheck` (or a Compose `command:`)
to run a different subcommand. The distroless `HEALTHCHECK` runs the binary's own
`healthcheck` subcommand, which GETs `/health` and exits 0/non-zero.

### Configuration

All settings can come from a YAML file (`--config`, or `STORAGE_PROXY_CONFIG`);
`--host` / `--port` / `--upstream-url` / `--upstream-token` overlay it, and a
config-less run works given `--upstream-url`. Secrets should be referenced from
the environment (`{ env: VAR }`) rather than inlined.

```yaml
# storage-proxy.yaml
host: 0.0.0.0        # default; all interfaces
port: 8080           # default
base_path: /storage-proxy   # byte-proxy mount; default
upstream:
  base_url: http://uc:8080/api/2.1/unity-catalog/   # required
  token: { env: UC_TOKEN }   # optional; absent = contact UC unauthenticated
auth:
  mode: anonymous    # or `reverse-proxy` (see below)
```

The standalone deployment always announces `storageAccess: "proxy"` at
`/capabilities` — it exists precisely to serve the byte-proxy.

### Inbound request auth

Because the proxy vends credentials on the caller's behalf, in a shared
deployment access should be attributed to a real user. Two modes:

- **`anonymous`** (default) — every request is anonymous. Use when the upstream
  UC itself uses anonymous auth, or for single-tenant / local development.
- **`reverse-proxy`** — trust a forwarded-identity header set by an upstream
  proxy that has *already* authenticated the caller (e.g. nginx / Envoy / an
  OAuth2 proxy forwarding `X-Forwarded-User`). Only safe when the proxy is not
  directly reachable, since a client could otherwise forge the header. Configure
  the header and missing-identity behavior:

  ```yaml
  auth:
    mode: reverse-proxy
    forwarded_user_header: x-forwarded-user   # default
    allow_missing_identity: false             # default: reject (401) if absent
  ```

This is the standalone equivalent of the server crate's `ReverseProxyAuthenticator`
(`auth.mode: reverse-proxy`).

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
