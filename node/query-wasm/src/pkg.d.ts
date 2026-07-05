// Ambient typing for the "query-wasm-pkg" bare specifier — the wasm-bindgen
// output of `just build-query-wasm` (crates/query-wasm/pkg/query_wasm.js,
// gitignored). The worker imports the BARE name so this package type-checks
// whether or not the artifact exists; node/app/vite.config.ts aliases the name
// to the real file in wasm-enabled builds (default builds alias the whole
// package to ./stub.ts and never bundle the worker).
//
// Keep the shapes in sync with crates/query-wasm/src/bindings.rs (the
// generated pkg/query_wasm.d.ts is the source of truth).
declare module "query-wasm-pkg" {
  /** Summary returned by `runQuery`. */
  export interface RunStats {
    chunks: number;
    rows: number;
    tableVersion: number;
  }

  /** The in-browser Unity Catalog query engine. */
  export class UcQueryEngine {
    constructor(baseUrl: string, opts?: { authToken?: string });
    runQuery(
      sql: string,
      opts: { limit?: number; catalog?: string; schema?: string },
      onBatch: (ipc: Uint8Array, numRows: number) => void,
    ): Promise<RunStats>;
    free(): void;
  }

  /** wasm-bindgen module init (fetches + instantiates the .wasm). */
  // biome-ignore lint/style/noDefaultExport: mirrors wasm-bindgen's generated module shape.
  export default function init(input?: unknown): Promise<unknown>;
}
