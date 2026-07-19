// The volume-oriented files surface the UC UI consumes. It sits on top of the
// low-level `FilesRunner` seam (runner.ts): a `FilesService` turns a volume path
// into directory pages, file bytes and metadata. Nothing here knows the
// transport — the registered runner does. Mirrors @open-lakehouse/query's
// types.ts and @open-lakehouse/log-query's types.ts.
//
// The DTOs are structurally aligned with hydrofoil's portal `FilesService`
// messages (portal/files/v1/svc.proto) — `DirectoryEntry`, `FileMetadata`,
// `DirectoryMetadata`, the paged `ListDirectoryContents` request/response and the
// `DownloadFile` streaming request/chunk — so a generated proto contract can drop
// in later. They are defined LOCALLY (no generated proto dependency in this
// package, matching log-query) with UI-friendly `number` timestamps/sizes rather
// than the wire `int64`/`bigint`.

/**
 * A single entry within a directory listing — the proto analog of
 * `portal.files.v1.DirectoryEntry`. `path` is the entry's absolute path,
 * `fileSize` is `0` for directories, and `lastModified` is epoch milliseconds.
 */
export interface DirectoryEntry {
  /** Absolute path of the entry (e.g. `/Volumes/c/s/v/dir/file.parquet`). */
  path: string;
  /** True when the entry is a subdirectory (contents rolled up). */
  isDirectory: boolean;
  /** Size in bytes; `0` for directories. */
  fileSize: number;
  /** Last-modified time in epoch milliseconds. */
  lastModified: number;
}

/**
 * Metadata describing a single file — the proto analog of an HTTP HEAD
 * (`portal.files.v1.FileMetadata`). `contentType` / `etag` are optional (the
 * store may not surface them).
 */
export interface FileMetadata {
  /** Absolute path of the file. */
  path: string;
  /** Size in bytes. */
  fileSize: number;
  /** Last-modified time in epoch milliseconds. */
  lastModified: number;
  /** MIME type, when the store surfaces it. */
  contentType?: string;
  /** Entity tag, when the store surfaces it. */
  etag?: string;
}

/**
 * Metadata describing a directory — the proto analog of
 * `portal.files.v1.DirectoryMetadata`.
 */
export interface DirectoryMetadata {
  /** Absolute path of the directory. */
  path: string;
  /** Last-modified time in epoch milliseconds. */
  lastModified: number;
}

/**
 * A request for one bounded page of a directory listing — the proto analog of
 * `ListDirectoryContentsRequest`. `pageToken` resumes a prior listing (the
 * `nextPageToken` from a {@link DirectoryPage}).
 */
export interface ListDirectoryRequest {
  /** Directory whose immediate children to list. */
  path: string;
  /** Page size; the runner/store applies its own cap when omitted. */
  maxResults?: number;
  /** Opaque continuation token from a previous page's `nextPageToken`. */
  pageToken?: string;
}

/**
 * One bounded page of a directory listing — the proto analog of
 * `ListDirectoryContentsResponse`. `nextPageToken` is present iff more pages
 * remain.
 */
export interface DirectoryPage {
  /** The entries in this page. */
  entries: DirectoryEntry[];
  /** Continuation token; absent when this is the last page. */
  nextPageToken?: string;
}

/**
 * A request to read a file (or a byte range of it) — the proto analog of
 * `DownloadFileRequest`. `offset`/`length` select a range; both omitted reads
 * the whole file.
 */
export interface ReadFileRequest {
  /** Path of the file to read. */
  path: string;
  /** Byte offset to start from (defaults to 0). */
  offset?: number;
  /** Number of bytes to read (defaults to the rest of the file). */
  length?: number;
}

/**
 * One chunk of a file read stream — the proto analog of a `DownloadFile` server
 * stream chunk. Chunks arrive in file order and are concatenated by the caller.
 */
export interface FileChunk {
  /** A slice of the file contents. */
  bytes: Uint8Array;
}

/**
 * Capability probe input: what the UI knows about a volume before browsing its
 * files. `volumeType` is the UC volume type (MANAGED / EXTERNAL); `storageScheme`
 * is the cloud scheme of the storage location (e.g. `abfss`, `gs`, `s3`) — the
 * wasm backend supports Azure + GCP only, so it reads this to gate.
 */
export interface FilesSupportsInput {
  /** UC volume type, e.g. `MANAGED` / `EXTERNAL`. */
  volumeType?: string;
  /** Cloud scheme of the volume's storage location, e.g. `abfss` / `gs` / `s3`. */
  storageScheme?: string;
}

// ---------------------------------------------------------------------------
// Phase 2 (writes) — DEFINED BUT NOT YET USED.
//
// These shapes exist so the write verbs on `FilesRunner` / `FilesService` have a
// stable contract to grow into, but v1 is read-only: no runner implements them,
// `canWrite()` is false, and nothing in this package calls them. They mirror the
// portal `UploadFileRequest` / `UploadFileResponse` messages.
// ---------------------------------------------------------------------------

/**
 * Phase 2 — a request to write (upload) a file. Not yet used: writes are
 * deferred. Mirrors `portal.files.v1.UploadFileRequest` collapsed to a single
 * unary request (the seam streams via the runner, not this DTO).
 */
export interface WriteFileRequest {
  /** Destination path. */
  path: string;
  /** The file contents to write. */
  bytes: Uint8Array;
  /** MIME type of the file (optional). */
  contentType?: string;
}

/**
 * Phase 2 — the result of a write. Not yet used. Mirrors
 * `portal.files.v1.UploadFileResponse`.
 */
export interface WriteFileResult {
  /** Path of the written file. */
  path: string;
  /** Size in bytes of the written file. */
  fileSize: number;
  /** Entity tag of the written file, when the store surfaces it. */
  etag?: string;
}

/**
 * The swappable files service the UC UI consumes. `listDirectory`/`readFile`/
 * `stat` cover the read surface; `supports` gates whether a Files tab should be
 * offered at all (e.g. Azure/GCP only under the wasm backend), and `canWrite`
 * gates write affordances (false in v1). Mirrors `QueryService` /
 * `LogQueryService`, minus the Arrow store (files aren't Arrow).
 */
export interface FilesService {
  /** List one bounded page of a directory's immediate children. */
  listDirectory(
    req: ListDirectoryRequest,
    signal?: AbortSignal,
  ): Promise<DirectoryPage>;
  /** Read a file (or byte range), draining the underlying stream into one buffer. */
  readFile(req: ReadFileRequest, signal?: AbortSignal): Promise<Uint8Array>;
  /** Read a file (or byte range) as a stream of chunks, in file order. */
  readFileStream(
    req: ReadFileRequest,
    signal?: AbortSignal,
  ): AsyncIterable<FileChunk>;
  /** Metadata for a single file (the proto analog of an HTTP HEAD). */
  stat(path: string, signal?: AbortSignal): Promise<FileMetadata>;
  /** Whether the registered backend can browse the given volume. */
  supports(x: FilesSupportsInput): boolean;
  /** Whether the registered backend supports writes (false in v1). */
  canWrite(): boolean;
}
