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
// pkg/query_wasm.d.ts is the source of truth).
declare module "query-wasm-pkg" {
  /** One bounded page of a directory listing (the @open-lakehouse/files
   *  `DirectoryPage`), as `listDirectory` resolves it. */
  export interface DirectoryPage {
    entries: Array<{
      path: string;
      isDirectory: boolean;
      fileSize: number;
      lastModified: number;
    }>;
    nextPageToken?: string;
  }

  /** Metadata for a single file (the @open-lakehouse/files `FileMetadata`), as
   *  `stat` resolves it. */
  export interface FileMetadata {
    path: string;
    fileSize: number;
    lastModified: number;
    contentType?: string;
    etag?: string;
  }

  /** The in-browser Unity Catalog volume-files engine. Reads volume files
   *  direct-to-cloud (Azure/GCP) over vended UC credentials. */
  export class UcFilesEngine {
    constructor(baseUrl: string, opts?: { authToken?: string });
    /** List one bounded page of a directory's immediate children. */
    listDirectory(
      path: string,
      opts: { maxResults?: number; pageToken?: string },
    ): Promise<DirectoryPage>;
    /** Read a file (or byte range), calling `onChunk` per body chunk in file
     *  order; resolves once the read completes. */
    readFile(
      path: string,
      opts: { offset?: number; length?: number },
      onChunk: (bytes: Uint8Array) => void,
    ): Promise<unknown>;
    /** Metadata for a single file (HTTP HEAD analog). */
    stat(path: string): Promise<FileMetadata>;
    free(): void;
  }

  /** wasm-bindgen module init (fetches + instantiates the .wasm). */
  // biome-ignore lint/style/noDefaultExport: mirrors wasm-bindgen's generated module shape.
  export default function init(input?: unknown): Promise<unknown>;
}
