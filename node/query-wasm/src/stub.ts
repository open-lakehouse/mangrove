// Build-time stand-in for `@open-lakehouse/query-wasm` in default builds.
//
// The real entry reaches (through ./worker.ts) the gitignored wasm-bindgen
// artifact under crates/query-wasm/pkg/, which only exists after
// `just build-query-wasm`. Default builds must not resolve it, so
// node/app/vite.config.ts aliases the package here unless
// VITE_ENABLE_WASM_QUERY=true. Registering nothing leaves the seam's throwing
// default runner in place, and `hasQueryRunner()` keeps the preview UI hidden.

import type { WasmQueryOptions } from "./index";

/** No-op: the wasm engine is not part of this build. */
export function registerWasmPreview(_options: WasmQueryOptions): void {}

/** No-op: the wasm engine is not part of this build. The log-query seam keeps
 *  its throwing default runner, so `hasLogQueryRunner()` stays false and the
 *  Delta-log tab stays hidden (unless a dev stub is registered separately). */
export function registerWasmLogPreview(_options: WasmQueryOptions): void {}
