#!/usr/bin/env bun
/**
 * Packs a workspace package exactly as js-release publishes it (`bun pm pack
 * --ignore-scripts`, so build it first) and lints the tarball: `publint
 * --strict` for package.json/exports problems and `attw` (Are The Types Wrong)
 * for type resolution. ESM-only packages (`"type": "module"`) are checked with
 * attw's `esm-only` profile, which ignores the CJS resolution modes.
 *
 * Usage:
 *   bun run scripts/pack-check.ts <package-dir>...
 */
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { $ } from "bun";

const ROOT = resolve(import.meta.dirname, "..");
const bin = (name: string) => join(ROOT, "node_modules", ".bin", name);

const dirs = process.argv.slice(2);
if (dirs.length === 0) {
  console.error("usage: bun run scripts/pack-check.ts <package-dir>...");
  process.exit(2);
}

for (const dir of dirs) {
  const pkgDir = resolve(ROOT, dir);
  const pkg = JSON.parse(readFileSync(join(pkgDir, "package.json"), "utf8"));
  const dest = mkdtempSync(join(tmpdir(), "pack-check-"));
  const tarball = (
    await $`bun pm pack --ignore-scripts --quiet --destination ${dest}`
      .cwd(pkgDir)
      .text()
  ).trim();
  console.log(`\n== ${pkg.name}@${pkg.version} (${tarball})`);
  await $`${bin("publint")} run --strict ${tarball}`;
  const profile = pkg.type === "module" ? ["--profile", "esm-only"] : [];
  await $`${bin("attw")} ${tarball} ${profile} --format table-flipped`;
}
