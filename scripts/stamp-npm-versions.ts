#!/usr/bin/env bun
/**
 * Writes every npm package's version into its package.json before packing, and
 * pins each `workspace:` dependency on a sibling to that sibling's version.
 * package.json carries 0.0.0 in git; the version lives in the package's crate
 * (node/client's NAPI crate, or a placeholder crate next to package.json),
 * which release-plz bumps.
 *
 * The pinning is what makes a published package installable: `bun pm pack`
 * resolves `workspace:` specs from the workspace versions recorded in bun.lock
 * (all 0.0.0), not from the siblings' package.json. Trestle's env-editor 0.0.4
 * shipped depending on `@open-lakehouse/ui-kit@0.0.0` that way. With concrete
 * specs, pack has nothing left to resolve. All packages are stamped, not only
 * the ones released this run, so a dependency on an unreleased sibling pins to
 * its current (published) version.
 *
 * With `--expect '<json>'` (js-release's `packages` output: `[{dir, version}]`),
 * also fails if a released package's crate version isn't the released one.
 *
 * Usage:
 *   bun run scripts/stamp-npm-versions.ts [--expect '<json>']
 */
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const ROOT = resolve(import.meta.dirname, "..");
const NODE = join(ROOT, "node");

/** The `[package]` version of the crate at `dir`, if it has a Cargo.toml. */
function crateVersion(dir: string): string | undefined {
  const manifest = join(dir, "Cargo.toml");
  if (!existsSync(manifest)) return undefined;
  const pkg = readFileSync(manifest, "utf8")
    .split(/^\[/m)
    .find((section) => section.startsWith("package]"));
  return pkg?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
}

const DEP_FIELDS = [
  "dependencies",
  "devDependencies",
  "peerDependencies",
  "optionalDependencies",
] as const;

/** A `workspace:` spec pinned to `version`: `*` exact, `^`/`~` keep their range. */
export function pinWorkspaceSpec(spec: string, version: string): string {
  const range = spec.slice("workspace:".length);
  if (range === "*" || range === "") return version;
  if (range === "^" || range === "~") return `${range}${version}`;
  // An explicit range (`workspace:^1.2.0`) is already concrete.
  return range;
}

const manifests = readdirSync(NODE, { withFileTypes: true }).flatMap(
  (entry) => {
    if (!entry.isDirectory()) return [];
    const dir = join(NODE, entry.name);
    const path = join(dir, "package.json");
    const version = crateVersion(dir);
    if (!version || !existsSync(path)) return [];
    const pkg = JSON.parse(readFileSync(path, "utf8"));
    return [{ dir: `node/${entry.name}`, path, pkg, version }];
  },
);
const versions = new Map<string, string>(
  manifests.map(({ pkg, version }) => [pkg.name, version]),
);

const stamped = new Map<string, string>();
for (const { dir, path, pkg, version } of manifests) {
  pkg.version = version;
  for (const field of DEP_FIELDS) {
    const deps: Record<string, string> | undefined = pkg[field];
    for (const [name, spec] of Object.entries(deps ?? {})) {
      if (!spec.startsWith("workspace:")) continue;
      const sibling = versions.get(name);
      if (!sibling) {
        console.error(
          `${pkg.name} depends on ${name} via ${spec}, which isn't a published package`,
        );
        process.exit(1);
      }
      (deps as Record<string, string>)[name] = pinWorkspaceSpec(spec, sibling);
    }
  }
  writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`);
  stamped.set(dir, version);
  console.log(`${pkg.name}@${version}`);
}

const expectAt = process.argv.indexOf("--expect");
if (expectAt !== -1) {
  const released: { dir: string; version: string }[] = JSON.parse(
    process.argv[expectAt + 1] ?? "[]",
  );
  for (const { dir, version } of released) {
    const actual = stamped.get(dir);
    if (actual !== version) {
      console.error(
        `${dir}: released ${version}, but its crate is at ${actual ?? "no version"}`,
      );
      process.exit(1);
    }
  }
}
