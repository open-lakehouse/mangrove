# Handover: browser/wasm storage-access strategy — same-origin byte-proxy

**Status:** Research + architecture decision only. **No implementation done.** This
document is the artifact; a later session picks it up to implement. All findings
below were verified against the codebase and the pinned `object_store` fork on
2026-07-23.

**One-line recommendation:** build a UC-server-agnostic **same-origin storage
byte-proxy** (read + write) whose client side is the `object_store` fork's stock
`HttpStore` + a custom `HttpConnector`, and whose server side is a new
self-contained crate mounted like `crates/delta-api`, config-gated, with a
server-level capabilities endpoint announcing the `storageAccess` posture.
Default posture = `proxy`; `direct` remains an opt-in fast path.

---

## Why (the problem)

The browser/wasm engine (DataFusion + delta-kernel on `wasm32`) reads Delta
tables **directly from cloud object storage** with a vended credential, and the
volume-files path writes the same way. This works **only for Azure** today and
requires the customer to configure **CORS** on the storage account.

Two independent browser gates — conflating them is the trap:
- **Auth gate** ("may this request touch this object?") — solved by the vended
  credential (`crates/server/src/services/credential_vending.rs`,
  `crates/server/src/rest/routers/mod.rs` temporary-credentials routes).
- **CORS gate** ("may this *origin* cross-origin-`fetch()` and read the response?")
  — satisfied **only** by CORS headers returned *by the storage host*. **A
  presigned/SAS URL does NOT avoid CORS** (verified against AWS S3 CORS docs;
  identical for GCS + Azure). So "just document CORS" is a per-cloud
  customer-config burden, not an Azure-vs-others distinction.

## User priorities & constraints (captured this session)
- **S3 is the most-requested cloud**; adoption gated on feasibility/effort.
- **Proxy is the default** posture for a fresh deployment.
- Proxy must **work against *any* Unity Catalog server**, not just mangrove.
- **`object_store` is the backend/integration seam** — the integration decides
  how a store is wired; the proxy stays UC-server-agnostic.
- Depend on **injected handlers for the UC API surfaces needed**, backed **either
  by a client (remote UC) or by direct server handlers** — the **hybrid** pattern.
- Reuse the **Open Sharing** integration pattern (`crates/sharing-api`).
- The **UC server exposes a server-level config/capabilities endpoint** (in
  scope) announcing `storageAccess`; extensible beyond the proxy.
- **Per-user auth identity is a parallel track**, not a blocker.
- **Volumes need write support** (uploads), not just read.
- The main artifact is this handover; **no implementation in the discovering
  session.**

## Decisive insight
A same-origin byte-proxy **makes S3/GCP work on wasm for free**: the wasm store
never speaks a cloud-native protocol, so we neither lift the AWS/GCP hard-reject
(`crates/object-store/src/lib.rs:680-686`) nor fight S3 SigV4/`Range` CORS
preflight. All cloud signing stays server-side where the native AWS/GCP/Azure
code already works. In-browser pushdown (projection, row-group pruning, Delta
data-skipping) runs first, so only *pruned byte ranges* transit the server.

---

## Verified facts (the evidence base)

### Current data path is direct-to-storage for BOTH consumers
The wasm `object_store` store is the shared bottom of two browser seams, both
CORS-gated on the cloud host today:
1. **Query/read** — DataFusion + kernel issue `get_opts` (Range GET) + `head`.
2. **Volume files** — `crates/query-wasm/src/files/engine.rs` `write_file` →
   `for_volume(…, ReadWrite)` → `store.put_opts(…, PutMode::Overwrite|Update)`;
   **writes go browser→Azure directly today.** File *metadata* (list/stat/delete/
   mkdir) rides a **separate ConnectRPC `FilesService`** (`files/service.rs`), NOT
   `object_store.list/delete` — so only the **bytes** (read+write) need the proxy.

### object_store fork capabilities (roeap `e4676ee`, pinned in `crates/query-wasm/Cargo.toml`)
Source: `~/.cargo/git/checkouts/arrow-rs-object-store-cba19ccf9cd1da4c/e4676ee/src`.
- Minimal read surface a Delta scan needs = **`get_opts(path, GetOptions{range,
  head, if_match})`** + `head`. `GetRange` = `Bounded|Suffix|FromOffset`.
- **`object_store::http::{HttpStore, HttpBuilder}` is NOT gated out on wasm.**
  `HttpStore.get_opts` = plain GET (+`Range`), `HttpStore.put_opts` = plain `PUT`
  **for `PutMode::Overwrite`** (non-Overwrite → `NotImplemented`; no multipart).
  `HttpStore.list` = WebDAV PROPFIND (we do NOT serve this).
- `HttpBuilder::with_http_connector(C: HttpConnector)` injects a custom transport.
  `HttpConnector::connect(&ClientOptions) -> HttpClient`; `HttpService::call(req)`
  is the wasm Fetch seam (the fetch-cache/no-store fix lives here).
- **AWS/GCP stores are `cfg(all(feature="cloud", not(wasm32)))`** — so an
  "S3-REST reverse-proxy reusing `AmazonS3Builder` on wasm" is **impossible**
  (SigV4 can't run in wasm anyway). No client reuse to be had that way.

### The two reusable in-repo patterns
- **`object_store` at the seam already exists in `sharing-api`:**
  `crates/sharing-api/src/kernel.rs:28`
  `trait ObjectStoreFactory { async fn create_object_store(&self, url:&Url) ->
  Result<Arc<DynObjectStore>>; }` — backend generic over it; satisfied by
  `InMemory` (tests) or a UC-client-backed factory. **This is the model.**
- **Injectable client-or-handler = the hybrid pattern:** router builders in
  `crates/server/src/rest/routers/mod.rs` are generic over a handler trait `T`;
  `crates/server/src/hybrid.rs build_hybrid_router` selects per surface
  `RoutingMode::Local => ServerHandler` vs `Upstream => UpstreamXHandler::new(
  policy, client.x_client())` (same trait over a **client** or **local handler**),
  chosen by `RoutingConfig` in `crates/server/src/config.rs`.
- **`UnityObjectStoreFactory` is already UC-server-agnostic:** built from just
  `{baseUrl, token}` (`builder().with_uri().with_token().build()`); `unity_client()`,
  `credentials_client()`, `for_table/for_volume/for_path(op) -> UCStore`;
  `UCStore::root()/as_dyn() -> Arc<dyn ObjectStore>` (`crates/object-store/src/lib.rs`).

### The mount pattern to mirror = `crates/delta-api` (NOT a proxy itself)
`olai-uc-delta-api`: a **backend port** (`DeltaBackend<Cx>`), a generic
blanket-impl handler, a **state-agnostic composable router**
(`router_with_context::<S,Cx>(handler, extract_cx) -> Router<S>`), a
capability-driven `getConfig`, and a feature-gated in-memory testing backend.
Server integration: adapter `crates/server/src/services/delta_backend.rs` impls
the port for `ServerHandler`; `create_delta_router` in
`crates/server/src/rest/routers/mod.rs:21` supplies the `Principal→RequestContext`
extractor; `build_rest_router` in `crates/server/src/run.rs` `.merge()`s it.
Server-level config endpoints today: only root `/health`,`/version`
(`run.rs:259`) + delta-v1 `getConfig` — **no server-wide capabilities endpoint yet.**

---

## Decisions made this session

1. **Primary approach: the same-origin byte-proxy.** (Not Envoy — see below.)
2. **Client shape: stock `HttpStore` + custom `HttpConnector`.** ~0 new client
   store code; abstraction lives at the HTTP transport/connector layer, reusing
   the existing `olai_http_wasm::WasmClient` Fetch transport + Bearer `with_auth`
   (`crates/object-store/src/lib.rs:240-265`). NOT a bespoke `ObjectStore` impl;
   NOT an S3-REST RP.
3. **Proxy covers read + write** (both consumers bottom out on object_store).
   File metadata stays on the ConnectRPC `FilesService`.
4. **Listing:** the Delta read path never lists; volume browsing lists via the
   existing FilesService. So the proxy does **not** serve WebDAV; if a list verb
   is ever wanted it's a small JSON endpoint, not `HttpStore.list()`.
5. **Server-level capabilities endpoint is in scope**, announces
   `storageAccess: "direct"|"proxy"` (default `proxy`), extensible.
6. **Auth identity (`ReverseProxyAuthenticator`) is a parallel follow-up**, built
   against the current `AnonymousAuthenticator` seam (`crates/server/src/rest/auth.rs`).

### Wire contract (WebDAV-free)
```
GET  /storage-proxy/{securable}/{key}            -> 200 + body + Content-Length + ETag
GET  /storage-proxy/{securable}/{key}  Range: .. -> 206 + Content-Range (+ Accept-Ranges)
HEAD /storage-proxy/{securable}/{key}            -> headers only
PUT  /storage-proxy/{securable}/{key}  [body]    -> 200/201 + ETag   (whole-object upload)
```
On PUT the server vends a **write-scoped** credential and streams the body to
storage. **Conditional-write wrinkle:** `HttpStore.put_opts` rejects non-Overwrite,
but the files path wants `if_match` (`PutMode::Update{etag}`). Preferred fix: the
wasm write path sends `Overwrite` + an `If-Match` **header** the proxy enforces
server-side (`412`→conflict). Decide during implementation.

### Client wasm factory branch (sketch)
```rust
// crates/object-store/src/lib.rs, wasm-only path in to_store/build_store (L548-692),
// taken when storageAccess == proxy:
let store = object_store::http::HttpBuilder::new()
    .with_url(format!("{base}/…/storage-proxy/{securable}/"))
    .with_http_connector(/* WasmClient-backed HttpConnector, Bearer via with_auth */)
    .with_retry(RetryConfig { max_retries: 0, ..Default::default() }) // wasm: no timer
    .build()?; // Arc<dyn ObjectStore>: serves get_opts + put_opts(Overwrite)
```
The AWS/GCP hard-reject (L680-686) drops off the hot path (the wasm store no
longer speaks cloud protocols) → **S3/GCP on wasm without lifting it**. Both
`catalog.rs` (read) and `files/engine.rs resolve_volume_rw` (write) build stores
from the factory, so both get the proxy store automatically.

---

## Alternative evaluated & rejected: Envoy `aws_request_signing` (edge SigV4)

Idea: deploy UC behind Envoy; use Envoy's `aws_request_signing` filter to
SigV4-sign S3 at the edge; mangrove exposes a dynamic (gRPC) credential provider
that vends a per-user credential from the session.

**Verdict: not viable as designed** (Envoy docs + config proto verified):
- The filter *does* sign S3 (SigV4/SigV4A) and S3 is its most-mature target.
- But **all** its credential providers are static/host-level (inline, env, config
  file, `assume_role_with_web_identity`, IAM Roles Anywhere, instance/container),
  cached ~1h. **No per-request, per-user, or gRPC/callout dynamic provider; no
  header/dynamic-metadata/filter-state/ext_proc/lua/wasm hook to inject creds per
  request.** So "mangrove gRPC provider forwards user identity → per-user scoped
  cred per request" **cannot be wired** — Envoy signs every request with one
  deployment-wide identity.

**The core architectural limitation (the real reason byte-proxy wins):** Envoy
edge-signing **decouples signing from authorization**. A config/SDS server can
supply the signing secret, but it's still a *deployment-wide* credential on a
timer — it changes *where* the secret lives, not the *per-session* fact. To
recover per-user/per-path safety you'd have to **re-express mangrove's authz as
external request-path rules** (OPA/ext_authz) in front of a broadly-privileged
signer — duplicating and risking drift from authz mangrove already computes. By
contrast the vending path **bakes scope into the credential itself**
(`build_s3_session_policy` → bucket+prefix STS session policy; path-scoped SAS):
**authorize + scope + sign is one atomic step.** The byte-proxy's local arm keeps
that (authorize at vend time, in-process, per request); Envoy structurally cannot.
Also: Envoy is AWS-only (Azure/GCP unaddressed) and adds heavy data-path infra.
Revisit only if per-request dynamic creds land in Envoy upstream.

Other rejected options: server `302 → presigned URL` (browser follows
cross-origin → CORS reappears); lifting the wasm AWS/GCP hard-reject for direct
S3-on-wasm (wasm S3/GCS builder forks + SigV4/`Range` preflight — most effort,
most customer config); CDN / S3 Object Lambda / Front Door (high-egress escape
valve, per-cloud customer config — document as advanced).

---

## Recommended architecture (for the implementing session)

A new self-contained **`crates/storage-proxy`** (`olai-uc-storage-proxy`),
`object_store` at the seam, mirroring `sharing-api`/`delta-api`:
- **Port:** reuse/extend `sharing-api`'s `ObjectStoreFactory` shape →
  `StorageProxyBackend<Cx>` with `capabilities()`, `authorize(&ProxyReq, &Cx)`,
  `open(&ProxyReq, &Cx) -> Arc<DynObjectStore>` (verb = Get/Head/Put; returns
  read- or write-scoped store). Handler translates to `get_opts`/`put_opts` and
  **streams** bodies (bounded memory), relaying `Range`/`206`/`Content-Range`/
  `ETag`, enforcing `If-Match`.
- **Router:** `router_with_context::<S,Cx>` (mirror `delta-api::router`).
- **Testing:** feature-gated in-memory backend over `object_store::memory::InMemory`.

### Two interchangeable backend arms (the "any UC server" requirement)
- **Client arm (portable):** wraps a `UnityObjectStoreFactory` built from
  `{baseUrl, token}`; `open` calls `for_table/for_volume/for_path(op)` →
  `UCStore::root()`. Zero mangrove coupling → works against any UC server.
- **Local arm (in-process):** `crates/server/src/services/storage_proxy_backend.rs`
  impls the port over `ServerHandler` + `credential_vending.rs`, enforcing `Policy`
  and vending read/ReadWrite creds per verb. Selected by config, exactly like
  `UpstreamXHandler` vs `ServerHandler` in `hybrid.rs`.

### Phasing (suggested)
1. `crates/storage-proxy` crate (port + handler + router + testing).
2. Backend arms (client arm in-crate; local arm in server).
3. Server mount, config-gated: `create_storage_proxy_router` in
   `rest/routers/mod.rs`; `storage_proxy` flag in `config.rs`; conditional
   `.merge()` in `run.rs build_rest_router` AND `hybrid.rs build_hybrid_router`.
4. Server-level capabilities endpoint announcing `storageAccess`.
5. wasm factory branch (above) + thread `storageAccess` from
   `crates/query-wasm/src/catalog.rs` and `files/engine.rs`; pass the hint through
   `node/query-wasm` and `node/files-wasm` worker protocols. Leave the query
   fallback (`node/query/src/fallback.ts`, `isFallbackWorthy`) and the ConnectRPC
   `FilesService` metadata path unchanged.
6. **Parallel follow-up:** `ReverseProxyAuthenticator` for per-user identity.
7. **Optional/later:** formalize `direct` fast path + document per-cloud CORS/CDN.

---

## Open risks for the implementing session
1. **Egress/cost** — even pruned, popular/wide previews move real bytes through
   the server. Measure bytes-through-server; know when CDN becomes necessary.
2. **Server memory** — stream with bounded buffers + concurrency limits; never
   buffer whole objects (read or write).
3. **Range end-to-end** — the wasm store must issue `Range` GETs, the proxy must
   forward them, `206`/`Content-Range` must relay back; a collapse to full-object
   GETs breaks the cost model.
4. **Confused-deputy** — server-side path-scope validation is mandatory for GET
   **and PUT** (writable proxy raises the stakes).
5. **Write path** — whole-object PUT only (no multipart); confirm largest upload
   streams within memory limits; resolve the `If-Match` conditional-write wrinkle.
6. **GCS vending gap** — `credential_vending.rs` implements Azure + AWS only;
   GCP-out-of-the-box waits on server-side GCS vending (client arm inherits this).
7. **Auth identity** — `AnonymousAuthenticator` default means proxied vends are
   anonymous/over-privileged until `ReverseProxyAuthenticator` lands; gate the
   proxy default on real identity in non-anonymous deployments.

## Verification (end-to-end, for the implementing session)
- **Unit (crate):** GET preserves `Range`→`206`/`Content-Range`; PUT relays
  `ETag`, enforces `If-Match`→`412`; rejects out-of-scope paths (GET+PUT); no
  full-object buffering. Feature-gated in-memory backend.
- **Integration (native):** server against **Azurite** (`object-store/src/lib.rs`
  Azurite handling) + **MinIO/S3 mock**; drive with a native `HttpStore`-shaped
  client doing ranged GETs + PUTs; assert read parity, write→read round-trip,
  only pruned ranges. Also point the **client arm at a second UC server** to prove
  UC-agnosticism.
- **wasm E2E:** query-wasm harness runs a Delta preview **and a volume upload +
  read-back** with `storageAccess:"proxy"` against Azurite **and S3-mock** — S3
  proves S3-on-wasm works *without* lifting the hard-reject; assert results match
  the native reference and the browser makes **zero cross-origin requests**.
- **Capabilities:** the server config endpoint flips `storageAccess` with the
  config flag and the wasm resolver honors it.
- **Fallback:** force proxy `NETWORK`/`FAILED` before first chunk → transparent
  re-run on the server-query runner; force `UNSUPPORTED` (deletion vectors) → same.
- **Pre-push (per CLAUDE.md):** `cargo fmt --all --check`; `cargo clippy
  --workspace --all-targets --all-features -- -D warnings`; `cargo nextest run
  --workspace`; doc tests.

---

## Key file references
- `crates/object-store/src/lib.rs` — `UnityObjectStoreFactory` (client arm); wasm
  store branch (L548-692); AWS/GCP hard-reject (L680-686); wasm transport/auth
  (L240-265); Azurite handling.
- `crates/sharing-api/src/kernel.rs:28` — `ObjectStoreFactory` port to reuse;
  `crates/sharing-api/src/{backend,handler,router,testing}.rs` — pattern.
- `crates/delta-api/src/{backend,handler,router,config}.rs` — mount pattern;
  `crates/server/src/services/delta_backend.rs` — adapter;
  `crates/server/src/rest/routers/mod.rs:21` — `create_delta_router`;
  `crates/server/src/run.rs` — `build_rest_router`; `crates/server/src/hybrid.rs`
  + `crates/server/src/handlers/upstream.rs` — client-or-handler selection.
- `crates/server/src/services/credential_vending.rs` — `vend_credential`,
  `build_s3_session_policy`, `vend_azure_sas_from_bearer`, `vend_aws_iam_role`,
  `vended_cache`.
- `crates/server/src/config.rs` — `Config` / `RoutingConfig` / `RoutingMode`.
- `crates/server/src/rest/auth.rs` — `AnonymousAuthenticator` (identity TODO).
- `crates/query-wasm/src/catalog.rs` (read); `crates/query-wasm/src/files/engine.rs`
  `write_file`/`resolve_volume_rw` (write); `crates/query-wasm/src/files/service.rs`
  (ConnectRPC metadata); `crates/query-wasm/src/bindings.rs` (`writeFileBytes`).
- `node/query/src/fallback.ts`, `node/query-wasm`, `node/files-wasm` — client seams.
- object_store fork `e4676ee`: `http/mod.rs` (`HttpStore`/`HttpBuilder`),
  `client/http/connection.rs` (`HttpConnector`/`HttpService`), `lib.rs`
  (`ObjectStore`/`GetOptions`/`PutMode`), `util.rs` (`GetRange`).

## Related memory / prior art
`files-wasm-connect-transport` (volume write path, PR #162),
`query-wasm-volume-factory-shipped` (VOLUME path on `for_volume`),
`wasm-fetch-cache-etag-fix` (HttpService no-store fix),
`olai-uc-client-wasm-shipped`, `delta-api-crate-extracted`,
`sharing-api-carveout`, `wasm-object-store-and-ring-toolchain`.
