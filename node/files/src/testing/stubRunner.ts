// A dev/stub `FilesRunner` that serves a rich, deterministic volume-files tree —
// no wasm, no server. It proves the files seam + the volume Files UI end-to-end
// and is the data the UI is built against until the real wasm runner lands.
//
// Lives on the `@open-lakehouse/files/testing` subexport (mirroring log-query's
// `./testing` and query-wasm's `./stub`) so it ships with the seam it satisfies
// and backs both the app's dev wiring and Storybook stories, without polluting
// the main barrel.

import { type FilesRunner, registerFilesRunner } from "../runner";
import type {
  DirectoryPage,
  FileChunk,
  FileMetadata,
  ReadFileRequest,
} from "../types";
import {
  fixtureFileChunks,
  fixtureFileMetadata,
  fixtureListing,
} from "./fixtures";

// Default page size when a request omits `maxResults` — small so the root's
// 15-entry listing arrives in several pages, exercising `nextPageToken`
// continuation.
const DEFAULT_PAGE_SIZE = 5;

// A page token is just the numeric offset into the directory's full listing,
// stringified — opaque to callers, but trivially resumable by the stub.
function parseOffset(token?: string): number {
  const n = token ? Number.parseInt(token, 10) : 0;
  return Number.isFinite(n) && n >= 0 ? n : 0;
}

/**
 * A stub runner serving the canned fixture tree (testing/fixtures.ts). Directory
 * listings are paged via a numeric-offset `nextPageToken`; `readFile` streams a
 * couple of canned chunks; `stat` returns the fixture's file metadata. Honours
 * abort. No write verbs (v1 is read-only), so `canWrite` stays false.
 */
export const stubFilesRunner: FilesRunner = {
  async listDirectory(req, { signal }): Promise<DirectoryPage> {
    signal.throwIfAborted();
    const all = fixtureListing(req.path);
    const size =
      req.maxResults && req.maxResults > 0 ? req.maxResults : DEFAULT_PAGE_SIZE;
    const offset = parseOffset(req.pageToken);
    const slice = all.slice(offset, offset + size);
    const nextOffset = offset + slice.length;
    return {
      entries: slice,
      nextPageToken: nextOffset < all.length ? String(nextOffset) : undefined,
    };
  },

  readFile(req: ReadFileRequest, { signal }): AsyncIterable<FileChunk> {
    return {
      async *[Symbol.asyncIterator](): AsyncIterator<FileChunk> {
        signal.throwIfAborted();
        for (const bytes of fixtureFileChunks(req.path)) {
          if (signal.aborted) throw signal.reason ?? new Error("aborted");
          yield { bytes } satisfies FileChunk;
        }
      },
    };
  },

  async stat(path, { signal }): Promise<FileMetadata> {
    signal.throwIfAborted();
    const meta = fixtureFileMetadata(path);
    if (!meta) throw new Error(`stub: no such file: ${path}`);
    return meta;
  },
};

/**
 * One-call wiring: register the stub as the app's files runner with a permissive
 * capability probe (the volume-type/scheme gate lives in the tab) and no write
 * support. Call once at startup, before the UI bootstraps. Mirrors
 * `registerStubLogPreview`.
 */
export function registerStubFiles(): void {
  registerFilesRunner(stubFilesRunner, {
    supports: () => true,
    canWrite: false,
  });
}
