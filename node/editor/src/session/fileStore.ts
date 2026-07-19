// The file-persistence seam the editor session reads/writes tabs through.
//
// The editor package defines the SHAPE; the host (e.g. the Unity Catalog UI over
// the Volumes Files API) provides the implementation and passes it as a prop to
// `EditorSessionProvider`. This is an injected dependency, NOT a global
// singleton: a file store is contextual (per-volume / per-scope), and the
// session provider is mounted per scope — a prop rides that lifetime naturally
// and is trivially swapped in tests/stories with an in-memory store.
//
// The store deals in bytes; the editor encodes/decodes text at its boundary
// (Monaco needs whole strings), so binary handling stays possible later.
//
// READ-ONLY MODE: `writeFile` is OPTIONAL. When the host omits it (a store that
// can only read — e.g. the current volume files backend, whose writes are
// deferred), the session runs read-only: tabs open and are editable in the
// buffer, but autosave is disabled (no dirty→save cycle, no persistence). The
// session exposes this via `isReadOnly`.

/** File metadata returned alongside a read (the proto/HTTP analog of a stat). */
export interface FileStat {
  /** Opaque version tag; used for write-if-match conflict detection. */
  etag: string;
}

export interface ReadResult {
  bytes: Uint8Array;
  stat: FileStat;
}

/** Byte-level file access the editor session reads tabs through, and — when the
 *  host supports writes — persists them through. `writeFile` absent ⇒ read-only. */
export interface FileStore {
  readFile(path: string): Promise<ReadResult>;
  /** Persist a file. OPTIONAL: omit for a read-only store (autosave disabled). */
  writeFile?(
    path: string,
    bytes: Uint8Array,
    opts: { ifMatchEtag?: string; contentType: string },
  ): Promise<{ etag: string }>;
}

/** Raised by a FileStore's writeFile when the `ifMatchEtag` precondition fails
 *  (the file changed underneath the buffer). Autosave surfaces this as a
 *  conflict rather than a generic error. */
export class ConflictError extends Error {
  constructor(message = "File changed on disk") {
    super(message);
    this.name = "ConflictError";
  }
}
