// Unit tests for formatFileSize — boundary rounding and unit selection.

import { expect, test } from "bun:test";
import { formatFileSize } from "./fileSize";

test("zero and sub-byte inputs collapse to 0 B", () => {
  expect(formatFileSize(0)).toBe("0 B");
  expect(formatFileSize(-1)).toBe("0 B");
  expect(formatFileSize(Number.NaN)).toBe("0 B");
  expect(formatFileSize(Number.POSITIVE_INFINITY)).toBe("0 B");
});

test("bytes render without a decimal", () => {
  expect(formatFileSize(1)).toBe("1 B");
  expect(formatFileSize(512)).toBe("512 B");
  expect(formatFileSize(1023)).toBe("1023 B");
});

test("exact unit boundaries roll over with one decimal", () => {
  expect(formatFileSize(1024)).toBe("1.0 KiB");
  expect(formatFileSize(1024 * 1024)).toBe("1.0 MiB");
  expect(formatFileSize(1024 * 1024 * 1024)).toBe("1.0 GiB");
});

test("intermediate values round to one decimal", () => {
  expect(formatFileSize(1536)).toBe("1.5 KiB"); // 1.5 KiB exactly
  expect(formatFileSize(2_621_440)).toBe("2.5 MiB"); // 2.5 MiB exactly
  // 4096 + 0*9137 + 0*1024 = 4096 → the fixture's file(…, 0) size.
  expect(formatFileSize(4096)).toBe("4.0 KiB");
});

test("scales through TiB/PiB", () => {
  expect(formatFileSize(1024 ** 4)).toBe("1.0 TiB");
  expect(formatFileSize(1024 ** 5)).toBe("1.0 PiB");
  // Beyond the table it stays in PiB rather than inventing a unit.
  expect(formatFileSize(1024 ** 6)).toBe("1024.0 PiB");
});
