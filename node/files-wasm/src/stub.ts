// Build-time stand-in for `@open-lakehouse/files-wasm` in default builds.
//
// The real entry reaches (through ./worker.ts) the gitignored wasm-bindgen
// artifact under crates/query-wasm/pkg/, which only exists after
// `just build-query-wasm`. Default builds must not resolve it, so
// node/app/vite.config.ts aliases the package here unless the wasm build is
// enabled. Registering nothing leaves the seam's throwing default runner in
// place, and `hasFilesRunner()` keeps the Files tab hidden.

import type { WasmFilesOptions } from "./index";

/** No-op: the wasm files engine is not part of this build. The files seam keeps
 *  its throwing default runner, so `hasFilesRunner()` stays false and the volume
 *  Files tab stays hidden (unless a dev stub is registered separately). */
export function registerWasmFiles(_options: WasmFilesOptions): void {}
