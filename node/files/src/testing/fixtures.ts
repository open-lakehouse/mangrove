// Rich, deterministic volume-files fixture data — the directory tree the volume
// Files UI is built against until the real wasm runner lands. It mirrors the
// portal `DirectoryEntry` / `FileMetadata` shapes (portal/files/v1/svc.proto):
// a nested tree of directories and files with mixed sizes and last-modified
// timestamps.
//
// Everything is derived deterministically (no Math.random) so stories and tests
// are stable, but with enough variation — nested subdirectories, differing file
// sizes and timestamps, and ENOUGH immediate children under the root that a small
// page size yields ≥2 pages (exercising `nextPageToken` continuation).

import type { DirectoryEntry, FileMetadata } from "../types";

/** The canonical root the fixture tree hangs under. */
export const FIXTURE_VOLUME_ROOT = "/Volumes/demo/raw/events";

// A stable epoch-ms base (2026-05-01T00:00:00Z) so timestamps are deterministic.
const BASE_MS = Date.UTC(2026, 4, 1, 0, 0, 0);

// One directory entry with a size/timestamp derived from its index, so the whole
// tree is reproducible.
function file(path: string, i: number): DirectoryEntry {
  return {
    path,
    isDirectory: false,
    fileSize: 4_096 + i * 9_137 + (i % 7) * 1_024,
    lastModified: BASE_MS + i * 3_600_000,
  };
}

function dir(path: string, i: number): DirectoryEntry {
  return {
    path,
    isDirectory: true,
    fileSize: 0,
    lastModified: BASE_MS + i * 86_400_000,
  };
}

// The fixture tree, keyed by absolute directory path -> its immediate children
// (in listing order: directories first, then files). The root has 3 subdirs + 12
// files = 15 entries, so a pageSize of e.g. 5 yields 3 pages.
function buildTree(): Map<string, DirectoryEntry[]> {
  const tree = new Map<string, DirectoryEntry[]>();
  const root = FIXTURE_VOLUME_ROOT;

  // Root: three date-partition subdirectories + a batch of loose files.
  const rootChildren: DirectoryEntry[] = [];
  const partitions = ["date=2026-05-01", "date=2026-05-02", "date=2026-05-03"];
  partitions.forEach((name, i) => {
    rootChildren.push(dir(`${root}/${name}`, i));
  });
  for (let i = 0; i < 12; i++) {
    rootChildren.push(
      file(`${root}/part-${String(i).padStart(5, "0")}.snappy.parquet`, i),
    );
  }
  tree.set(root, rootChildren);

  // Each partition holds a `_metadata` subdir + a handful of parquet files.
  partitions.forEach((name, p) => {
    const base = `${root}/${name}`;
    const children: DirectoryEntry[] = [dir(`${base}/_metadata`, p)];
    for (let i = 0; i < 4; i++) {
      children.push(
        file(`${base}/chunk-${String(i).padStart(3, "0")}.parquet`, p * 10 + i),
      );
    }
    tree.set(base, children);

    // The `_metadata` leaf holds two small files (a deeper nesting level).
    tree.set(`${base}/_metadata`, [
      file(`${base}/_metadata/schema.json`, p),
      file(`${base}/_metadata/manifest.json`, p + 1),
    ]);
  });

  return tree;
}

const TREE = buildTree();

/**
 * The immediate children of a fixture directory, in listing order. Returns an
 * empty array for an unknown (or leaf-file) path — a stub `listDirectory` treats
 * that as an empty directory.
 */
export function fixtureListing(path: string): DirectoryEntry[] {
  return TREE.get(normalize(path)) ?? [];
}

/**
 * File metadata for a fixture path, derived from the entry in its parent
 * directory's listing. Returns `null` for an unknown path or a directory.
 */
export function fixtureFileMetadata(path: string): FileMetadata | null {
  const norm = normalize(path);
  for (const children of TREE.values()) {
    const entry = children.find((e) => e.path === norm && !e.isDirectory);
    if (entry) {
      return {
        path: entry.path,
        fileSize: entry.fileSize,
        lastModified: entry.lastModified,
        contentType: guessContentType(entry.path),
        etag: `"etag-${entry.fileSize.toString(16)}"`,
      };
    }
  }
  return null;
}

/**
 * Deterministic canned file contents for a fixture path — a couple of chunks of
 * bytes so a stub `readFile` streams more than one `FileChunk`. Derived from the
 * path length so it is stable but path-dependent.
 */
export function fixtureFileChunks(path: string): Uint8Array[] {
  const seed = normalize(path).length;
  const first = Uint8Array.from({ length: 8 }, (_, i) => (seed + i) & 0xff);
  const second = Uint8Array.from(
    { length: 4 },
    (_, i) => (seed * 2 + i) & 0xff,
  );
  return [first, second];
}

// Strip a single trailing slash so `/Volumes/.../events/` matches
// `/Volumes/.../events`.
function normalize(path: string): string {
  return path.length > 1 && path.endsWith("/") ? path.slice(0, -1) : path;
}

function guessContentType(path: string): string {
  if (path.endsWith(".json")) return "application/json";
  if (path.endsWith(".parquet")) return "application/vnd.apache.parquet";
  return "application/octet-stream";
}
