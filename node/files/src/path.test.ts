// Unit tests for the canonical /Volumes path helpers (path.ts) — parse
// round-trips, rejection of non-/Volumes paths, nested rest + trailing/duplicate
// slashes, case-insensitive `Volumes`, and the fullName / join helpers.

import { expect, test } from "bun:test";
import {
  formatVolumePath,
  joinVolumePath,
  parseVolumePath,
  volumeFullName,
} from "./path";

test("parses a canonical volume path with a nested rest", () => {
  const vp = parseVolumePath("/Volumes/cat/sch/vol/a/b/c.parquet");
  expect(vp).toEqual({
    catalog: "cat",
    schema: "sch",
    volume: "vol",
    relativePath: "a/b/c.parquet",
  });
});

test("parses a volume root (no rest) to an empty relativePath", () => {
  const vp = parseVolumePath("/Volumes/cat/sch/vol");
  expect(vp).toEqual({
    catalog: "cat",
    schema: "sch",
    volume: "vol",
    relativePath: "",
  });
});

test("format round-trips a parsed path to the canonical string", () => {
  const input = "/Volumes/cat/sch/vol/a/b/c.parquet";
  const vp = parseVolumePath(input);
  if (!vp) throw new Error("expected a parsed volume path");
  expect(formatVolumePath(vp)).toBe(input);
});

test("format of a root path omits the trailing slash", () => {
  expect(
    formatVolumePath({
      catalog: "c",
      schema: "s",
      volume: "v",
      relativePath: "",
    }),
  ).toBe("/Volumes/c/s/v");
});

test("is case-insensitive on the leading Volumes segment", () => {
  for (const prefix of ["/volumes", "/VOLUMES", "/VoLuMeS"]) {
    const vp = parseVolumePath(`${prefix}/c/s/v/x`);
    if (!vp) throw new Error(`expected ${prefix} to parse`);
    expect(vp.relativePath).toBe("x");
    // Formatting always normalizes back to the canonical capitalized prefix.
    expect(formatVolumePath(vp)).toBe("/Volumes/c/s/v/x");
  }
});

test("tolerates trailing and duplicate slashes", () => {
  const vp = parseVolumePath("/Volumes//cat/sch//vol/a//b/");
  expect(vp).toEqual({
    catalog: "cat",
    schema: "sch",
    volume: "vol",
    relativePath: "a/b",
  });
});

test("rejects non-/Volumes and too-short paths", () => {
  expect(parseVolumePath("/catalog/sch/vol/x")).toBeNull();
  expect(parseVolumePath("/Volumes/cat/sch")).toBeNull(); // missing volume
  expect(parseVolumePath("/Volumes")).toBeNull();
  expect(parseVolumePath("")).toBeNull();
  expect(parseVolumePath("relative/path")).toBeNull();
});

test("volumeFullName renders the dotted three-level name", () => {
  const vp = parseVolumePath("/Volumes/cat/sch/vol/x/y");
  if (!vp) throw new Error("expected a parsed volume path");
  expect(volumeFullName(vp)).toBe("cat.sch.vol");
});

test("joinVolumePath appends segments onto a volume base", () => {
  expect(joinVolumePath("/Volumes/c/s/v", "a", "b")).toBe("/Volumes/c/s/v/a/b");
  // Extends an existing rest and normalizes casing.
  expect(joinVolumePath("/volumes/c/s/v/a", "b/c")).toBe(
    "/Volumes/c/s/v/a/b/c",
  );
  // Collapses extra slashes in segments.
  expect(joinVolumePath("/Volumes/c/s/v", "/a/", "/b/")).toBe(
    "/Volumes/c/s/v/a/b",
  );
});

test("joinVolumePath on a non-volume base still joins cleanly", () => {
  expect(joinVolumePath("/some/dir", "a", "b")).toBe("/some/dir/a/b");
  expect(joinVolumePath("rel", "a")).toBe("rel/a");
});
