# Agent Instructions

Instructions for AI coding agents working in this repository. This is the single
canonical file: `CLAUDE.md` imports it, and Cursor/Codex read it directly. Human
contributor detail (prerequisites, release mechanics, PyO3 stubs, service layout)
lives in [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md).

## Project map

Multi-crate Rust workspace with language bindings:

- `crates/` — `common` (shared models; mostly generated proto code, feature-gated
  per resource), `handlers` (generated per-resource handler traits + axum routes),
  `client`, `server`, `cli`, `delta-api`, `files-api` / `files-proto`,
  `storage-proxy`, `storage-access` (credentials + external-locations store port),
  `object-store`, `datafusion`, `postgres`, `sqlite`, `acceptance`
- `wasm/` — a **separate** Cargo workspace (own `Cargo.lock`) for the in-browser
  query engine
- `python/client/` — PyO3 bindings · `node/` — NAPI client, TS client, UI packages
  (see [`node/README.md`](node/README.md))
- `proto/` — API definitions (source of truth) · `openapi/` — generated specs ·
  `dev/` — docker-compose, scripts, Marimo notebooks
- Open Sharing / Delta Sharing serving lives in
  [`distributary`](https://github.com/open-lakehouse/distributary); the
  shares/recipients/providers models stay here. Cloud auth/HTTP and storage come
  from the external trestle crates (`olai-http`, `olai-store`, …).

## Commands

[`just`](https://just.systems/) recipes live in modules; call them namespaced
(`just --list` shows the tree). A few common ones also work flat via aliases.

| Command | Purpose |
|---|---|
| `just generate` | Full codegen pipeline (needs `trestle` + a `../trestle` checkout) |
| `just dev rest` / `just dev rest-db` | Dev server (in-memory SQLite / Postgres) |
| `just meta fix` / `just meta fmt` | Auto-fix lint, format (Rust, Node, Python) |
| `just meta lint-node` | Lint + typecheck the `node/` workspace |
| `just build build-py` / `just build build-node` | Build Python / Node bindings |
| `just test test-node` / `just test test-node-integration` | Node tests |
| `just dev configure-trestle-deps` / `just dev lock-registry` | Patch trestle crates from the local checkout / restore crates.io lock entries (see below) |

The fast edit/check loop (`cargo check`, `cargo test`) needs none of the codegen
tooling.

## Hard rules

- **Never hand-edit generated files**: anything under `crates/**/codegen/**`,
  `crates/**/models/gen/**`, `crates/**/models/_gen/**`, or starting with
  `// @generated`. Change the proto/codegen input, regenerate, and keep the
  generated output in the same commit as its source change. Python stubs
  (`python/client/python/unitycatalog_client/_client/*.pyi`) are generated too, by
  `just codegen generate-py-stubs` (see CONTRIBUTING).
- **Never commit a `Cargo.lock` that still resolves trestle crates from a local
  patch.** CI resolves them from crates.io. Run `just dev lock-registry` to
  restore the crates.io entries (`just dev check-lock` and the pre-commit hook
  check it). Never commit a `[patch.crates-io]` or `.cargo/config.toml`.
- **Never bump versions or edit changelogs by hand** (crates or `package.json`);
  release-plz does it from the Release PR.
- **Never commit to `main`.** Stage files by name — never `git add -A` / `git add .`.

## Testing

- Changes ship with tests; never delete or comment out a passing test to green a
  suite — fix the code or update the test to the new correct behavior.
- **Rust:** unit tests in `#[cfg(test)]` modules, integration tests in `tests/` and
  `crates/acceptance/`; `rstest` for parameterized cases. Run
  `cargo nextest run --workspace --all-features`, plus
  `cargo test --workspace --all-features --doc` (nextest skips doctests).
- **TypeScript:** `node/client` uses Jest; the other `node/` packages use
  `bun:test` with co-located `*.test.ts`.
- Cover error and edge paths, not just the happy path. Control non-deterministic
  inputs (time, random IDs). Coverage is reported by CI (Codecov, informational) —
  use it as guidance, not a gate.

## Style

Rust: `cargo fmt` + `cargo clippy` (edition/MSRV in the workspace `Cargo.toml`).
TS/JS: Biome, 2-space indent. Python: Ruff, 100-char lines. `typos` runs in
pre-commit. Trim trailing whitespace; end files with a newline.

## Branches, PRs and stacks

- **PRs are squash-merged. The PR title is the commit release-plz sees** — a
  conventional commit (`feat` `fix` `refactor` `docs` `test` `chore` `perf` `build`
  `ci`, `!` for breaking) with the affected crate's short name as scope, e.g.
  `feat(client): …`. CI lints it (`pr-title.yml`). Commits inside a PR are for
  review only.
- **One coherent change = one PR.** When work should land on `main` as *separate*
  commits — several crates or change types that each deserve their own changelog
  entry/bump — use a stack, one layer per `type(scope)`.
- **Stacks** use the [`gh stack`](https://github.com/github/gh-stack) extension:
  `gh stack init <base>` adopts a branch, `gh stack add <branch>` creates the next
  layer, `gh stack submit` pushes and links the PRs (each layer targets the one
  below), `gh stack sync --prune` cascades after a lower layer merges,
  `gh stack rebase` / `gh stack modify` restructure. Prefer these over raw
  `git rebase --onto` + `gh pr edit --base`, which leaves stack tracking stale.
- **Create branches server-side first.** The push-time secret scanner checks the
  full ancestry of a branch's first push and trips on old false positives in
  `main`'s history. Create the ref remotely, then fetch and track it:

  ```bash
  gh api repos/open-lakehouse/mangrove/git/refs -f ref=refs/heads/<branch> \
    -f sha="$(git rev-parse origin/main)"
  git fetch origin <branch> && git switch --track origin/<branch>
  ```

  For a stack layer, use the parent layer's tip as `sha`.
- PR body: what changed and why, a test plan, and `Closes #N` when an issue exists.
- **Re-trigger CI with an empty commit** (`git commit --allow-empty`), never by
  closing/reopening the PR.

### Pre-push check (mirrors CI)

```bash
cargo fmt --all --check \
  && cargo clippy --workspace --all-targets --all-features -- -D warnings \
  && cargo nextest run --workspace --all-features --profile ci \
       -E 'not (package(olai-uc-postgres) and (binary(commit_coordinator) or binary(storage_access)))' \
  && cargo test --workspace --all-features --doc
```

## Builds and disk

Dev builds keep line tables for workspace crates and no debug info for
dependencies (`[profile.dev]` in both workspaces). Panic locations and backtrace
file:line for workspace code are kept. For a debugger, use
`cargo build --profile debugging` (full debug info, separate `target/debugging/`).
Run `cargo clean` in a worktree before removing it.

## trestle crates and the crates.io proxy

The local crates.io proxy hides releases for 7 days, so a fresh `olai-*` release
from trestle won't resolve locally at first. Patch from the trestle checkout
(`just dev configure-trestle-deps`), build, then run `just dev lock-registry`
before committing. An unreleased trestle change gets a committed git pin
(`{ version, git, rev }`) instead. Both strategies are described in trestle's
[`docs/crates-proxy.md`](https://github.com/open-lakehouse/trestle/blob/main/docs/crates-proxy.md).
The recipes find trestle next to the main mangrove checkout (worktrees too) or
via `TRESTLE_ROOT`.

## Environment

Some targets need services or credentials (a local `.env` is dotenv-loaded):
`just dev rest-db` / `just build build-sqlx` need Docker + Postgres;
`just test record-managed` needs `DATABRICKS_HOST`, `DATABRICKS_TOKEN` and
`DATABRICKS_STORAGE_ROOT`. Enable the repo git hooks once per clone with
`bun run setup-hooks`.
