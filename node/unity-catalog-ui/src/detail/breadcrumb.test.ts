// Validates the breadcrumb crumb-target derivation used in VolumeEditor's
// Breadcrumb component — the pure path math, exercised without React. The
// component builds each crumb target with joinVolumePath(root, ...segsUpToHere),
// so this asserts that formula against the exported @open-lakehouse/files helpers.

import { expect, test } from "bun:test";
import { joinVolumePath, parseVolumePath } from "@open-lakehouse/files";

// Mirror the derivation in Breadcrumb: the volume crumb → root, then one crumb
// per relativePath segment, each targeting the path up to and including it.
function crumbTargets(root: string, currentPath: string): string[] {
  const vp = parseVolumePath(currentPath);
  const segments = vp ? vp.relativePath.split("/").filter(Boolean) : [];
  return [
    root,
    ...segments.map((_, i) =>
      joinVolumePath(root, ...segments.slice(0, i + 1)),
    ),
  ];
}

const ROOT = "/Volumes/demo/raw/events";

test("root path yields a single crumb (the volume) targeting the root", () => {
  expect(crumbTargets(ROOT, ROOT)).toEqual([ROOT]);
});

test("nested path yields a crumb per segment, each cumulative", () => {
  const current = `${ROOT}/date=2026-05-01/_metadata`;
  expect(crumbTargets(ROOT, current)).toEqual([
    ROOT,
    `${ROOT}/date=2026-05-01`,
    `${ROOT}/date=2026-05-01/_metadata`,
  ]);
});

test("the parent (up) target drops the last segment", () => {
  const current = `${ROOT}/date=2026-05-01/_metadata`;
  const vp = parseVolumePath(current);
  const segments = vp ? vp.relativePath.split("/").filter(Boolean) : [];
  expect(joinVolumePath(ROOT, ...segments.slice(0, -1))).toBe(
    `${ROOT}/date=2026-05-01`,
  );
  // From a first-level dir, up returns to the root.
  expect(joinVolumePath(ROOT, ...["date=2026-05-01"].slice(0, -1))).toBe(ROOT);
});
