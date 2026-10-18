#!/usr/bin/env bun
/**
 * Builds a publishable workspace package into `dist/`: `tsc -p
 * tsconfig.build.json` emits ESM JS + declarations (+ maps), then the source
 * files tsc does not emit are copied across with their paths kept: hand-written
 * `.d.ts` files (e.g. a generated `uc-api.d.ts`, ambient module declarations),
 * and `.json` / `.css` assets. Tests and stories are skipped.
 *
 * Run from the package directory (each package's `build` script does):
 *   bun run ../../scripts/build-package.ts
 */
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { $, Glob } from "bun";

const pkgDir = process.cwd();
const dist = join(pkgDir, "dist");

rmSync(dist, { recursive: true, force: true });
await $`tsc -p tsconfig.build.json`.cwd(pkgDir);

const assets = new Glob("**/*.{d.ts,json,css}");
const skip = /\.(test|stories)\.[^/]+$/;
for await (const rel of assets.scan({ cwd: join(pkgDir, "src") })) {
  if (skip.test(rel)) continue;
  const to = join(dist, rel);
  mkdirSync(dirname(to), { recursive: true });
  cpSync(join(pkgDir, "src", rel), to);
}
