# Agent Instructions

Instructions for AI coding agents (Claude Code, Cursor, Codex, Copilot, etc.) working
in this repository.

## How rules are organized

| Location | Purpose |
|----------|---------|
| **`AGENTS.md`** (this file) | Portable, harness-agnostic agent instructions |
| **[`CLAUDE.md`](CLAUDE.md)** | Full repository guidelines — structure, build commands, codegen, PR workflow |
| **[`.agents/rules/`](.agents/rules/)** | Canonical always-applied rules (`.mdc` files with YAML frontmatter) |
| **[`.cursor/rules/`](.cursor/rules/)** | Cursor-specific copies of selected rules (when present) |

**Precedence:** explicit user chat instructions override everything; then the closest
`AGENTS.md` in the directory tree; then `.agents/rules/*.mdc` for always-applied
constraints.

When a rule exists in both this file and `.agents/rules/`, treat
**`.agents/rules/` as the source of truth** and keep this file in sync when rules
change.

---

## Always-applied rules

These rules apply to every agent session. Full text lives in `.agents/rules/`.

### 1. Branching workflow

**File:** [`.agents/rules/branching-rule.mdc`](.agents/rules/branching-rule.mdc)

- **Never commit directly to `main`.** Create a feature branch before the first file change.
- Before starting work, check the current branch. If on `main`, create and switch to a new branch.
- Suggest a branch name using `feat/<short-description>` and offer the user a choice (suggested name or their own).
- Branch prefixes:

  | Prefix | Use when |
  |--------|----------|
  | `feat/` | Adding new functionality |
  | `fix/` | Fixing a bug |
  | `chore/` | Build, CI, dependency, or repo upkeep |
  | `refactor/` | Restructuring without behavior change |
  | `test/` | Adding or improving tests only |

- One logical unit of work per branch. Keep names short, lowercase, and hyphen-separated.

### 2. Testing discipline

**File:** [`.agents/rules/testing-discipline.mdc`](.agents/rules/testing-discipline.mdc)

**Core principles:**

1. Every change ships with tests — no production code is complete without test coverage.
2. Never delete a passing test. Fix the code or update the test to match correct behavior.
3. Target ~80% coverage across methods and conditional branches.

**Rust**

- Unit tests in `#[cfg(test)]` modules; integration/acceptance in `tests/` or `crates/acceptance/`.
- Use `rstest` for parameterized tests.
- Run: `cargo nextest run --workspace --all-features`
- Doctests: `cargo test --workspace --doc` (nextest does not run them).
- Coverage:

  ```bash
  cargo llvm-cov nextest --workspace --all-features --lcov --output-path lcov.info
  cargo llvm-cov --doc --workspace --all-features --lcov --output-path lcov.info
  ```

**Go**

- Table-driven tests with the standard `testing` package (`foo.go` → `foo_test.go`).
- Go has no native `--lcov` flag. Collect and convert:

  ```bash
  go test ./... -coverprofile=coverage.out
  go install github.com/jandelgado/gcov2lcov@latest
  gcov2lcov -infile=coverage.out -outfile=lcov.info
  ```

- Local review: `go tool cover -func=coverage.out`

**TypeScript**

- Use `bun:test`. Co-locate tests (`foo.ts` → `foo.test.ts`) or use `__test__/`.
- Coverage:

  ```bash
  bun test --coverage --coverage-reporter=lcov
  # or: bun run test:coverage
  ```

  Writes `coverage/lcov.info`.

**All languages:** upload LCOV output via the Codecov GitHub Action. Do not skip flaky tests, delete tests to green a suite, or assert on non-deterministic values without controlling inputs.

### 3. bun.lock proxy stripping

**File:** [`.agents/rules/bun-clean-lock-rule.mdc`](.agents/rules/bun-clean-lock-rule.mdc)

Local Bun installs may bake a private npm registry/proxy URL into the resolution
(second) field of `bun.lock`. **Never open a PR with those URLs committed.**

Before any PR that touches `bun.lock`:

```bash
bun run strip-lock-proxy
bun run strip-lock-proxy:check
```

- Strip logic is host-agnostic — clear any `http(s)` registry tarball URL in the resolution slot, leaving `""`.
- Only the resolution field is modified; names, versions, dependency maps, and integrity hashes are untouched.
- A pre-commit hook (`.githooks/pre-commit`) strips proxy URLs when `bun.lock` is staged. Activate once per clone: `bun run setup-hooks`.

---

## Repository guidelines

See **[`CLAUDE.md`](CLAUDE.md)** for the full guide. Key points agents should know:

### Project structure

Multi-crate Rust workspace (`crates/`) with Python (`python/client/`), Node/TypeScript
(`node/`), protobuf (`proto/`), and generated code throughout. Use `just` as the
primary task runner.

### Code generation

**Never hand-edit generated files** — anything under `crates/**/codegen/**`,
`crates/**/models/_gen/**`, files marked `// @generated`, etc. Change proto/codegen
inputs and regenerate with `just generate`; commit generated output in the same commit.

### Common commands

| Command | Purpose |
|---------|---------|
| `just generate` | Full code generation pipeline |
| `just rest` | Start development REST server |
| `just test-node` | Run Node.js binding tests |
| `just fix` | Auto-fix Rust and Node.js code issues |
| `just fmt` | Format all code |
| `cargo nextest run --workspace --all-features` | Run Rust test suite |
| `bun run test:coverage` | TypeScript tests with LCOV coverage |

### Pre-push check (mimics CI)

```bash
cargo fmt --all --check \
  && cargo clippy --workspace --all-targets --all-features -- -D warnings \
  && cargo nextest run --workspace --all-features --profile ci -E 'not binary(commit_coordinator)' \
  && cargo test --workspace --all-features --doc
```

### Pull requests

- Work on a feature branch (`feat/`, `fix/`, etc.) — never `main`.
- Generated code goes in the same commit as the source change that produced it.
- Strip `bun.lock` proxy URLs before opening a PR (see rule above).
- PR title: `<type>: <description> (#<issue>)`. Include test plan, `Closes #N`, and follow-up refs.

### Commit workflow

Commit **unsigned** as you go; sign the whole branch once before opening a PR.
See `.claude/skills/commit/SKILL.md` for the commit skill. Only create commits when
the user asks.

---

## Maintaining these instructions

When adding or changing an always-applied rule:

1. Add or update the `.mdc` file in `.agents/rules/`.
2. Mirror to `.cursor/rules/` if the rule should apply in Cursor with `alwaysApply`.
3. Update this `AGENTS.md` summary so harnesses that only read the root file stay current.
