// Canonical `/Volumes/<catalog>/<schema>/<volume>/<rest>` path helpers — the
// UI-side analog of hydrofoil's `VolumePath` and mangrove's `UCReference`. Pure
// (no I/O, no runner), so they are cheap to use anywhere and exhaustively
// unit-tested (path.test.ts).
//
// A UC volume path names a volume (`catalog`, `schema`, `volume`) plus a
// `relativePath` into it. The leading `Volumes` segment is matched
// case-insensitively (mirroring hydrofoil's `VolumePath` parse), but everything
// formatted back out uses the canonical capitalized `/Volumes` prefix.

/** A parsed `/Volumes/<catalog>/<schema>/<volume>/<rest>` path. */
export interface VolumePath {
  /** The catalog segment. */
  catalog: string;
  /** The schema segment. */
  schema: string;
  /** The volume segment. */
  volume: string;
  /** The path relative to the volume root — no leading slash, `""` at the root. */
  relativePath: string;
}

// Split a path into non-empty segments, tolerating leading/trailing/duplicate
// slashes (`/Volumes//c/s/v/` -> ["Volumes","c","s","v"]).
function segments(path: string): string[] {
  return path.split("/").filter((s) => s.length > 0);
}

/**
 * Parse a canonical `/Volumes/<catalog>/<schema>/<volume>/<rest>` path. The
 * leading `Volumes` segment is matched case-insensitively. Returns `null` for any
 * non-`/Volumes` path or one missing the catalog/schema/volume triple. Trailing
 * slashes and duplicate slashes are tolerated; the `relativePath` is the joined
 * remainder (no leading/trailing slash).
 */
export function parseVolumePath(path: string): VolumePath | null {
  const parts = segments(path);
  if (parts.length < 4) return null;
  if (parts[0]?.toLowerCase() !== "volumes") return null;
  const [, catalog, schema, volume, ...rest] = parts;
  if (!catalog || !schema || !volume) return null;
  return { catalog, schema, volume, relativePath: rest.join("/") };
}

/**
 * Format a {@link VolumePath} back to its canonical string, always using the
 * capitalized `/Volumes` prefix. A blank `relativePath` yields the volume root
 * (`/Volumes/<c>/<s>/<v>`).
 */
export function formatVolumePath(vp: VolumePath): string {
  const base = `/Volumes/${vp.catalog}/${vp.schema}/${vp.volume}`;
  const rel = segments(vp.relativePath);
  return rel.length > 0 ? `${base}/${rel.join("/")}` : base;
}

/**
 * The volume's fully-qualified three-level name, `catalog.schema.volume` — the
 * form UC APIs (e.g. temporary-volume-credentials) address a volume by.
 */
export function volumeFullName(vp: VolumePath): string {
  return `${vp.catalog}.${vp.schema}.${vp.volume}`;
}

/**
 * Join path segments onto a base `/Volumes/...` path, collapsing extra slashes.
 * Each segment may itself contain slashes (they are re-split). Returns the
 * canonical joined path; the base's `/Volumes` prefix casing is normalized when
 * it parses as a volume path, else the raw base is used.
 */
export function joinVolumePath(base: string, ...segs: string[]): string {
  const extra = segs.flatMap((s) => segments(s));
  const vp = parseVolumePath(base);
  if (vp) {
    const rel = [...segments(vp.relativePath), ...extra];
    return formatVolumePath({ ...vp, relativePath: rel.join("/") });
  }
  // Non-volume base: still join cleanly, preserving a leading slash if present.
  const baseSegs = segments(base);
  const joined = [...baseSegs, ...extra].join("/");
  return base.startsWith("/") ? `/${joined}` : joined;
}
