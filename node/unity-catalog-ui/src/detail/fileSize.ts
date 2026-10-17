// Human-readable byte-size formatting for the volume Files explorer's rows.
//
// A UI presentation concern local to unity-catalog — NOT part of the
// @open-lakehouse/files seam contract (that ships raw `fileSize: number` bytes).
// Kept in its own module (rather than inline in VolumeEditor.tsx) so it's
// unit-testable without a DOM. Promote it to the seam only if a second consumer
// appears.

const UNITS = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"] as const;

/**
 * Format a byte count as a compact binary-prefixed string: `0 B`, `512 B`,
 * `1.0 KiB`, `2.5 MiB`, … Bytes render without a decimal; larger units keep one.
 * Negative or non-finite inputs collapse to `0 B`.
 */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  // Bytes are always whole; larger units get one decimal place.
  const rendered = unit === 0 ? String(Math.round(value)) : value.toFixed(1);
  return `${rendered} ${UNITS[unit]}`;
}
