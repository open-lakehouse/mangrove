// A `FileStore` for the editor session, backed by the Unity Catalog Volumes
// Files API.
//
// This is the concrete implementation of the editor's `FileStore` seam. It is
// injected as a prop into `EditorSessionProvider` (not registered globally) so
// it can be scoped per volume/environment.
//
// ⚠️ GAP: mangrove's UC server does NOT yet expose a volume Files API — the
// generated OpenAPI client (`unity-catalog-client`) has `/volumes/{name}`
// (metadata) and `/temporary-volume-credentials`, but no file read/write. So
// this is bespoke `fetch` code against the STANDARD UC/Databricks Files API
// shape (`GET|PUT|HEAD /api/2.0/fs/files/<path>`, ETag / If-Match for
// write-if-match). Until the server implements it, use `memoryFileStore` (below)
// to exercise the editor session end-to-end. Validate the exact path/verbs/etag
// semantics against the running server before relying on `createUcFileStore`.

import { ConflictError, type FileStore } from "@open-lakehouse/editor/session";

/**
 * Build a `FileStore` over the UC Files API.
 *
 * @param filesApiBase e.g. `${window.location.origin}/api/2.0/fs/files` — the
 *   Files API root, distinct from the UC REST root (`/api/2.1/unity-catalog`).
 */
export function createUcFileStore(filesApiBase: string): FileStore {
  const url = (path: string) =>
    `${filesApiBase}/${path.replace(/^\/+/, "").split("/").map(encodeURIComponent).join("/")}`;

  return {
    async readFile(path) {
      const res = await fetch(url(path), { method: "GET" });
      if (!res.ok) {
        throw new Error(`readFile ${path} failed: ${res.status}`);
      }
      const bytes = new Uint8Array(await res.arrayBuffer());
      // The Files API returns an ETag header for write-if-match.
      const etag = res.headers.get("ETag") ?? "";
      return { bytes, stat: { etag } };
    },
    async writeFile(path, bytes, opts) {
      const headers: Record<string, string> = {
        "Content-Type": opts.contentType,
      };
      if (opts.ifMatchEtag) headers["If-Match"] = opts.ifMatchEtag;
      const res = await fetch(url(path), {
        method: "PUT",
        headers,
        body: bytes as unknown as BodyInit,
      });
      // 409 Conflict / 412 Precondition Failed → the file changed underneath us.
      if (res.status === 409 || res.status === 412) {
        throw new ConflictError();
      }
      if (!res.ok) {
        throw new Error(`writeFile ${path} failed: ${res.status}`);
      }
      const etag = res.headers.get("ETag") ?? "";
      return { etag };
    },
  };
}

/**
 * An in-memory `FileStore` — for stories, tests, and exercising the editor
 * session while the server-side Files API is still missing. Seed it with initial
 * file contents; writes bump a monotonic etag so write-if-match is testable.
 */
export function memoryFileStore(seed: Record<string, string> = {}): FileStore {
  const files = new Map<string, { bytes: Uint8Array; etag: string }>();
  const enc = new TextEncoder();
  let counter = 0;
  for (const [path, text] of Object.entries(seed)) {
    files.set(path, { bytes: enc.encode(text), etag: `v${counter++}` });
  }

  return {
    async readFile(path) {
      const f = files.get(path);
      if (!f) {
        // Treat unknown paths as new empty files so "open" always succeeds.
        const empty = { bytes: new Uint8Array(), etag: `v${counter++}` };
        files.set(path, empty);
        return { bytes: empty.bytes, stat: { etag: empty.etag } };
      }
      return { bytes: f.bytes, stat: { etag: f.etag } };
    },
    async writeFile(path, bytes, opts) {
      const existing = files.get(path);
      if (opts.ifMatchEtag && existing && existing.etag !== opts.ifMatchEtag) {
        throw new ConflictError();
      }
      const etag = `v${counter++}`;
      files.set(path, { bytes, etag });
      return { etag };
    },
  };
}
