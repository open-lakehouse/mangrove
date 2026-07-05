# @open-lakehouse/query-wasm

The in-browser query engine (`crates/query-wasm`: DataFusion + Delta on
`wasm32`, reading cloud storage directly with UC-vended credentials) packaged
as a `QueryRunner` for the [`@open-lakehouse/query`](../query) seam. See
`WASM_QUERY_PREVIEW.md` at the repo root for the full design.

## How it plugs in

`registerWasmPreview({ baseUrl })` registers a runner that spawns a **Web
Worker per preview run** (the wasm engine's synchronous kernel bursts must not
run on the main thread), with a **Delta-only** capability probe wired into
`QueryService.supports`. Aborting a run terminates its worker — the engine
holds no cross-run state.

Failures carry a machine-readable `code`:

| code          | meaning                                              | typical handling |
| ------------- | ---------------------------------------------------- | ---------------- |
| `UNSUPPORTED` | outside the engine's v1 envelope (deletion vectors, non-classic checkpoints, unbackfilled managed commits, AWS/R2 storage, zstd/brotli parquet) | fall back        |
| `NETWORK`     | direct storage fetch blocked (CORS or connectivity)  | fall back        |
| `FAILED`      | a real error                                         | surface          |

Pair `isFallbackWorthy` with `createFallbackQueryRunner` (from
`@open-lakehouse/query`) to compose the wasm engine with a host's
server-backed runner.

## Build gating

The worker imports the wasm-bindgen artifact under `crates/query-wasm/pkg/`,
which is **gitignored** and produced by `just build-query-wasm`. Default app
builds must not resolve it, so `node/app/vite.config.ts` aliases this whole
package to [`src/stub.ts`](src/stub.ts) (a no-op `registerWasmPreview`) unless
`VITE_ENABLE_WASM_QUERY=true`. `just ui-build-wasm` builds the engine and the
SPA with both the engine and the preview UI flag enabled.
