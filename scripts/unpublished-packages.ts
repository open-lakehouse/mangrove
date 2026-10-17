#!/usr/bin/env bun
/**
 * Lists the publishable workspace packages whose current version is not yet on
 * their registry — the JS release pipeline's "is a release due?" signal.
 *
 * This replaces gating on the Changesets action's `has-changesets == 'false'`,
 * which is also false at rest (a push with no release-worthy commit). Comparing
 * each package's version on `main` against its registry is correct in every
 * state: right after the Version Packages PR merges the bumped versions are
 * missing (=> publish), and at rest everything is already there (=> no-op).
 *
 * A package is publishable when it is not `private` and not in the Changesets
 * `ignore` list. Its registry is `publishConfig.registry` (default npmjs). Auth
 * comes from the ambient npm config (in CI, the `.npmrc` setup-node writes).
 *
 * Output: a JSON array `[{ name, version, dir }]` on stdout. With `--github-output`
 * also writes `packages=<json>`, `any=<bool>` and `napi=<bool>` (whether the NAPI
 * package is due, i.e. the multi-arch build must run) to `$GITHUB_OUTPUT`.
 *
 * Usage:
 *   bun run scripts/unpublished-packages.ts [--github-output]
 */
import { appendFileSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { $ } from "bun";

const ROOT = resolve(import.meta.dirname, "..");
const NAPI_PACKAGE = "@open-lakehouse/unity-catalog-node";
const DEFAULT_REGISTRY = "https://registry.npmjs.org";

interface Due {
  name: string;
  version: string;
  dir: string;
}

const readJson = (path: string) => JSON.parse(readFileSync(path, "utf8"));

const rootPkg = readJson(join(ROOT, "package.json"));
const ignored = new Set<string>(
  readJson(join(ROOT, ".changeset", "config.json")).ignore ?? [],
);

async function isPublished(
  name: string,
  version: string,
  registry: string,
): Promise<boolean> {
  const res =
    await $`npm view ${`${name}@${version}`} version --registry ${registry}`
      .quiet()
      .nothrow();
  if (res.exitCode === 0) return res.stdout.toString().trim() === version;
  const err = res.stderr.toString();
  // E404 is "not published yet" (for a brand-new package as well as a new
  // version). Anything else (auth, network) must fail loudly instead of
  // silently skipping or re-publishing.
  if (/E404|404 Not Found/.test(err)) return false;
  throw new Error(`npm view ${name}@${version} failed:\n${err}`);
}

const due: Due[] = [];
for (const dir of rootPkg.workspaces as string[]) {
  const pkg = readJson(join(ROOT, dir, "package.json"));
  if (pkg.private || ignored.has(pkg.name)) continue;
  const registry = pkg.publishConfig?.registry ?? DEFAULT_REGISTRY;
  if (!(await isPublished(pkg.name, pkg.version, registry))) {
    due.push({ name: pkg.name, version: pkg.version, dir });
  }
}

const json = JSON.stringify(due);
console.log(json);

if (process.argv.includes("--github-output")) {
  const out = process.env.GITHUB_OUTPUT;
  if (!out) throw new Error("--github-output requires $GITHUB_OUTPUT");
  appendFileSync(
    out,
    [
      `packages=${json}`,
      `any=${due.length > 0}`,
      `napi=${due.some((p) => p.name === NAPI_PACKAGE)}`,
      "",
    ].join("\n"),
  );
}
