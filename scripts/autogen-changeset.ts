#!/usr/bin/env bun
/**
 * Generates Changesets `.changeset/*.md` entries from conventional commits.
 *
 * This is the "automatic" half of the JS release pipeline (the counterpart to
 * release-plz reading commits for the Rust crates): instead of asking every
 * contributor to hand-write a changeset, CI runs this over the commits merged
 * since the last release and derives the per-package bumps from the conventional
 * commit `type(scope):` prefix.
 *
 *   type  -> bump    scope -> package
 *   feat  -> minor   the commit `(scope)` is matched against each workspace
 *   fix   -> patch   package: its directory basename, its unscoped npm name,
 *   perf  -> patch   and a few aliases (see scopeToPackages). A commit with no
 *   refactor -> patch scope, or a scope that matches no package, is skipped.
 *   others  -> skip  A `!` or `BREAKING CHANGE` footer promotes the bump to major.
 *
 * Only NON-private, non-ignored workspace packages are eligible — the same set
 * Changesets itself would version. Private packages (all `@open-lakehouse/*`
 * today) are skipped, matching the dormant publish scope.
 *
 * A hand-written changeset already present for a package in this batch wins:
 * this script never overwrites `.changeset/*.md` files it did not author, and
 * `changeset version` merges a manual + generated bump by taking the highest.
 *
 * Usage:
 *   bun run scripts/autogen-changeset.ts --since <git-ref>   # since a ref (exclusive)
 *   bun run scripts/autogen-changeset.ts                     # since the last release tag, else HEAD~1
 *   bun run scripts/autogen-changeset.ts --dry-run           # print, do not write
 */
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const ROOT = resolve(import.meta.dirname, "..");
const CHANGESET_DIR = join(ROOT, ".changeset");

type Bump = "major" | "minor" | "patch";
const BUMP_RANK: Record<Bump, number> = { patch: 0, minor: 1, major: 2 };

// Conventional-commit type -> default bump. Types absent here (chore, docs,
// test, ci, build, style) do not trigger a release on their own.
const TYPE_BUMP: Record<string, Bump> = {
  feat: "minor",
  fix: "patch",
  perf: "patch",
  refactor: "patch",
};

// --- workspace discovery ---------------------------------------------------

interface Pkg {
  name: string;
  dir: string; // basename under node/
  private: boolean;
}

function readWorkspacePackages(): Pkg[] {
  const rootPkg = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"));
  const globs: string[] = rootPkg.workspaces ?? [];
  const pkgs: Pkg[] = [];
  for (const g of globs) {
    // All workspace entries are concrete `node/<dir>` paths (no globs today),
    // but tolerate a trailing `/*` by expanding one directory level.
    if (g.endsWith("/*")) {
      const base = g.slice(0, -2);
      for (const entry of readdirSync(join(ROOT, base), {
        withFileTypes: true,
      })) {
        if (entry.isDirectory()) pkgs.push(loadPkg(join(base, entry.name)));
      }
    } else {
      pkgs.push(loadPkg(g));
    }
  }
  return pkgs;
}

function loadPkg(relDir: string): Pkg {
  const manifest = JSON.parse(
    readFileSync(join(ROOT, relDir, "package.json"), "utf8"),
  );
  return {
    name: manifest.name,
    dir: relDir.split("/").pop() as string,
    private: manifest.private === true,
  };
}

// Build scope -> package-names index. A scope may map to multiple packages
// (rare), so we collect a set. Aliases: the directory basename, the unscoped
// npm name (part after `/`), and the full npm name.
function buildScopeIndex(pkgs: Pkg[]): Map<string, Set<string>> {
  const index = new Map<string, Set<string>>();
  const add = (scope: string, name: string) => {
    const key = scope.toLowerCase();
    if (!index.has(key)) index.set(key, new Set());
    index.get(key)?.add(name);
  };
  for (const p of pkgs) {
    add(p.dir, p.name);
    add(p.name, p.name);
    const unscoped = p.name.includes("/") ? p.name.split("/")[1] : p.name;
    add(unscoped, p.name);
  }
  return index;
}

// --- commit parsing --------------------------------------------------------

interface ParsedCommit {
  type: string;
  scopes: string[];
  breaking: boolean;
  subject: string;
}

// Matches `type(scope): subject` / `type!: subject` / `type: subject`.
// A scope may be comma-separated (`feat(query,editor): ...`).
const HEADER = /^(\w+)(?:\(([^)]*)\))?(!)?:\s*(.+)$/;

function parseCommit(subjectLine: string, body: string): ParsedCommit | null {
  const m = HEADER.exec(subjectLine.trim());
  if (!m) return null;
  const [, type, scopeRaw, bang, subject] = m;
  const scopes = (scopeRaw ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  const breaking = Boolean(bang) || /^BREAKING[ -]CHANGE:/m.test(body);
  return { type: type.toLowerCase(), scopes, breaking, subject };
}

// --- git ------------------------------------------------------------------

function git(args: string[]): string {
  return execFileSync("git", args, { cwd: ROOT, encoding: "utf8" }).trim();
}

function resolveSince(explicit?: string): string {
  if (explicit) return explicit;
  // Prefer the most recent tag reachable from HEAD; fall back to HEAD~1.
  try {
    return git(["describe", "--tags", "--abbrev=0"]);
  } catch {
    return "HEAD~1";
  }
}

// Return commits (subject + body) in `since..HEAD` that touched node/**.
function commitsTouchingNode(
  since: string,
): { subject: string; body: string }[] {
  const SEP = ""; // record separator
  const FIELD = ""; // field separator
  const raw = git([
    "log",
    `${since}..HEAD`,
    "--no-merges",
    `--format=%s${FIELD}%b${SEP}`,
    "--",
    "node/",
  ]);
  if (!raw) return [];
  return raw
    .split(SEP)
    .map((r) => r.trim())
    .filter(Boolean)
    .map((r) => {
      const [subject, body = ""] = r.split(FIELD);
      return { subject: subject.trim(), body: body.trim() };
    });
}

// --- main ------------------------------------------------------------------

const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const sinceIdx = args.indexOf("--since");
const sinceArg = sinceIdx >= 0 ? args[sinceIdx + 1] : undefined;

const pkgs = readWorkspacePackages();
const eligible = new Set(pkgs.filter((p) => !p.private).map((p) => p.name));
const scopeIndex = buildScopeIndex(pkgs);

const since = resolveSince(sinceArg);
const commits = commitsTouchingNode(since);

// Accumulate the highest bump per eligible package, plus a summary line.
const bumps = new Map<string, { bump: Bump; summaries: string[] }>();
let skippedNoScope = 0;
let skippedUnmatched = 0;

for (const { subject, body } of commits) {
  const parsed = parseCommit(subject, body);
  if (!parsed) continue;
  const baseBump = TYPE_BUMP[parsed.type];
  if (!baseBump && !parsed.breaking) continue; // chore/docs/test/ci/etc.
  const bump: Bump = parsed.breaking ? "major" : baseBump;
  if (parsed.scopes.length === 0) {
    skippedNoScope++;
    continue;
  }
  for (const scope of parsed.scopes) {
    const names = scopeIndex.get(scope.toLowerCase());
    if (!names) {
      skippedUnmatched++;
      continue;
    }
    for (const name of names) {
      if (!eligible.has(name)) continue; // private / ignored
      const cur = bumps.get(name);
      if (!cur || BUMP_RANK[bump] > BUMP_RANK[cur.bump]) {
        bumps.set(name, {
          bump,
          summaries: [...(cur?.summaries ?? []), parsed.subject],
        });
      } else {
        cur.summaries.push(parsed.subject);
      }
    }
  }
}

if (bumps.size === 0) {
  console.log(
    `No release-worthy node/ commits in ${since}..HEAD ` +
      `(skipped: ${skippedNoScope} scopeless, ${skippedUnmatched} unmatched-scope). No changeset written.`,
  );
  process.exit(0);
}

// One aggregated changeset file for this batch. `changeset version` still merges
// it with any hand-written entries by taking the highest bump per package.
const frontmatter = [...bumps.entries()]
  .map(([name, { bump }]) => `"${name}": ${bump}`)
  .join("\n");
const summaryLines = [
  ...new Set([...bumps.values()].flatMap((b) => b.summaries)),
].map((s) => `- ${s}`);
const content = `---\n${frontmatter}\n---\n\n${summaryLines.join("\n")}\n`;

if (dryRun) {
  console.log(`# would write .changeset/auto-<sha>.md\n\n${content}`);
  process.exit(0);
}

// Name the file after the HEAD sha so re-running on the same tip is idempotent
// (overwrites its own prior output rather than piling up duplicates).
const headSha = git(["rev-parse", "--short", "HEAD"]);
const outPath = join(CHANGESET_DIR, `auto-${headSha}.md`);
writeFileSync(outPath, content);
console.log(
  `Wrote ${outPath} bumping ${bumps.size} package(s): ` +
    [...bumps.entries()].map(([n, b]) => `${n}=${b.bump}`).join(", "),
);
