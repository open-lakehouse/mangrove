// Ambient typing for the "query-wasm-pkg" bare specifier — the wasm-bindgen
// output of `just build-query-wasm` (crates/query-wasm/pkg/query_wasm.js,
// gitignored). The worker imports the BARE name so this package type-checks
// whether or not the artifact exists; node/app/vite.config.ts aliases the name
// to the real file in wasm-enabled builds (default builds alias the whole
// package to ./stub.ts and never bundle the worker).
//
// This is the SAME artifact @open-lakehouse/query-wasm's worker imports — the
// files engine (`UcFilesEngine`) ships in the same crate/pkg as `UcQueryEngine`.
// Only the declarations this package uses are repeated here.
//
// Keep the shapes in sync with crates/query-wasm/src/bindings.rs (the generated
// pkg/query_wasm.d.ts is the source of truth). `UcFilesEngine` now speaks the
// `portal.files.v1.FilesService` contract: metadata RPCs go through the single
// `connectUnary` binary-proto dispatch, and file bytes bypass it via the native
// `readFileBytes` / `writeFileBytes` calls.
declare module "query-wasm-pkg" {
  /** The in-browser Unity Catalog volume-files engine. Reads/writes volume files
   *  direct-to-cloud (Azure/GCP) over vended UC credentials. */
  export class UcFilesEngine {
    constructor(baseUrl: string, opts?: { authToken?: string });
    /** Dispatch one unary `portal.files.v1.FilesService` RPC through the in-wasm
     *  connect Router, as binary proto. `path` is the full RPC path; resolves to
     *  the binary-proto response body. */
    connectUnary(path: string, requestBytes: Uint8Array): Promise<Uint8Array>;
    /** Read a file (or byte range), calling `onChunk` per body chunk in file
     *  order; resolves once the read completes. Bytes bypass the dispatcher. */
    readFileBytes(
      path: string,
      opts: { offset?: number; length?: number },
      onChunk: (bytes: Uint8Array) => void,
    ): Promise<unknown>;
    /** Write (create or overwrite) a file from one buffered body. Bytes bypass
     *  the dispatcher. Resolves to `{ path, fileSize, etag? }`. */
    writeFileBytes(
      path: string,
      bytes: Uint8Array,
      opts: { contentType?: string; ifMatchEtag?: string },
    ): Promise<{ path: string; fileSize: number; etag?: string }>;
    free(): void;
  }

  /** wasm-bindgen module init (fetches + instantiates the .wasm). */
  export default function init(input?: unknown): Promise<unknown>;
}
