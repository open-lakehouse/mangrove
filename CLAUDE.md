# Repository Guidelines

## Project Structure & Module Organization

The project follows a multi-crate Rust workspace architecture with language bindings:

- `crates/` - Core Rust implementation organized by functionality:
  - `common/` - Shared types and utilities (most of its size is generated proto code under `models/gen/`)
  - `client/` - Unity Catalog client library
  - `server/` - REST API server implementation
  - `postgres/` - PostgreSQL backend integration
  - `cli/` - Command-line interface
  - `object-store/` - Cloud object storage abstraction (AWS, Azure, GCP)
  - `sharing-client/` - Delta Sharing protocol client
  - `acceptance/` - Acceptance testing utilities
- Cloud auth/HTTP and storage abstractions come from the external `olai-http` /
  `olai-store` crates (published from the [`trestle`](https://github.com/open-lakehouse/trestle)
  codegen repo). See [Code generation](#code-generation).
- `python/client/` - Python bindings via PyO3
- `node/client/` - Node.js bindings via NAPI
- `proto/` - Protocol buffer definitions
- `openapi/` - Generated OpenAPI specifications
- `dev/` - Docker Compose config, development scripts, and Marimo notebooks
- `docs/` - Astro-based documentation site
- `examples/` - Type-checked code examples (Rust, Python, TypeScript) injected into docs

## Build, Test, and Development Commands

We use [`just`](https://just.systems/) as the primary task runner. Key commands:

**Code generation** (requires the `../trestle` sibling checkout — see [Code generation](#code-generation)):
- `just generate` - Run complete code generation pipeline
- `just generate-code` - Run REST server/client code generation
- `just generate-proto` - Run code generation pipeline for Protocol Buffers
- `just generate-node` - Generate types for Node.js client
- `just generate-openapi` - Update OpenAPI specification

**Development server:**
- `just rest` - Start development REST server
- `just rest-db` - Start server with PostgreSQL backend

**Building:**
- `just build-py` - Build Python bindings
- `just build-node` - Build Node.js bindings
- `just build-docker` - Build Docker image
- `just build-sqlx` - Build SQLx queries for offline mode

**Testing:**
- `just test-node` - Run Node.js binding tests
- `just test-node-integration` - Run Node.js integration tests (starts a UC server)
- `just integration` - Run integration tests with mocked responses
- `just integration-record` - Record integration tests against a live server

**Code quality:**
- `just fix` - Auto-fix Rust and Node.js code issues
- `just fix-rust` - Auto-fix Rust code only
- `just fix-node` - Auto-fix Node.js code only
- `just fix-py` - Auto-fix Python code only
- `just lint-node` - Lint JavaScript/TypeScript code
- `just fmt` - Format all code (Rust, protobuf, etc.)

**Documentation:**
- `just docs` - Start the Astro documentation development server
- `just validate-examples` - Type-check all examples and build docs (Rust/Python/TypeScript)

Standard Rust commands also work:
- `cargo test` - Run Rust unit tests
- `cargo build` - Build Rust workspace
- `cargo clippy` - Run Rust linter

## Code generation

Much of the code is generated from the proto definitions in `proto/`. The fast
edit/check loop (`cargo check`, `cargo test`) does **not** require any of the
generation tooling — you only need it when changing proto or regenerating.

**Prerequisites (only needed to run `just generate*`):**
- The codegen tool lives in a **sibling `../trestle` checkout** — clone it
  adjacent to this repo (`just generate-code` / `generate-openapi` shell out to
  `../trestle/crates/trestle`). Without it, those targets fail with a
  "manifest path does not exist" error.
- `buf` (proto compiler), `uv` (Python), and `bun`/`bunx` (OpenAPI bundling).

**Commands:** `just generate` runs the full pipeline; `just generate-proto`,
`generate-code`, `generate-openapi`, and `generate-node` run individual stages.

**Never hand-edit generated files.** This includes anything under
`crates/**/codegen/**`, `crates/**/models/gen/**`, `crates/**/models/_gen/**`,
or any file beginning with `// @generated`. Change the proto/codegen inputs and
regenerate instead, then commit the generated output in the **same commit** as
the source change. The one hand-maintained exception is the spliced PyO3 stub
supplement (`python/client/_client_supplement.pyi`) — see `.github/CONTRIBUTING.md`.

## Environment & services

Some targets need external services or env vars (a local `.env` is dotenv-loaded):
- `just rest-db` / `build-sqlx` — require Docker + Postgres.
- `just integration` / `integration-record` — require `DATABRICKS_HOST`,
  `DATABRICKS_TOKEN`, and `DATABRICKS_STORAGE_ROOT`.

## Code Examples in Documentation

Docs use a snippet injection system. The `examples/` directory contains type-checked
source files for Rust, Python, and TypeScript. In MDX docs, use `<CodeExample snippet="name" />`
to render a tabbed code block for all three languages automatically.

**Snippet tagging convention** (tag lines are stripped from the rendered output):
- Rust/TypeScript: `// [snippet:name]` ... `// [/snippet:name]`
- Python: `# [snippet:name]` ... `# [/snippet:name]`

Snippet names use `snake_case` matching the Rust method name (e.g., `list_catalogs`).

**Adding a new example:**
1. Add the tagged region to `examples/rust/src/*.rs`, `examples/python/*.py`, and `examples/typescript/examples/*.ts`
2. Add the new file imports to `docs/src/components/CodeExample.astro` and include them in `buildRegistry()`
3. Verify with `just validate-examples`
4. Use `<CodeExample snippet="your_snippet_name" />` in any MDX doc page (import the component first: `import CodeExample from "../../../components/CodeExample.astro"`)

**The `CodeExample` Astro component** (`docs/src/components/CodeExample.astro`) uses Vite `?raw`
imports to load snippet files at build time and expands `<CodeExample />` to Starlight
`<Tabs syncKey="language">` blocks — so users only need to select their language once site-wide.
It throws a hard build error if a snippet name is not found, ensuring docs always stay in sync.
Tabs are synced across the entire site via Starlight's built-in `localStorage` persistence.

## Coding Style & Naming Conventions

**Rust**: Follow standard Rust conventions with 4-space indentation.
Use `cargo fmt` for formatting and `cargo clippy` for linting.
Edition and MSRV are pinned in the workspace `Cargo.toml` — read them there.

**JavaScript/TypeScript**: 2-space indentation, managed by Biome formatter/linter.

**Python**: 4-space indentation, 100-character line limit, enforced by Ruff.

**General**:
- Use descriptive names following each language's conventions
- Trim trailing whitespace and ensure final newline
- Proto definitions use snake_case, generated Rust uses PascalCase/snake_case appropriately

Pre-commit hooks enforce formatting with Biome, Ruff, and typos checking.

## Tooling

- **`uv`** - Python package manager; manages the `python/*` UV workspace
- **`bun`** - JavaScript/TypeScript package manager for the npm workspaces
- **Maturin** - Builds Python wheels from PyO3 Rust crates
- **`buf`** - Protocol buffer compiler; config in `buf.yaml` / `buf.gen.yaml`
- **NAPI** - Node.js native addon bindings for Rust
- **Marimo** - Reactive Python notebook environment; notebooks live in `dev/`

## Testing Guidelines

**Framework**: Uses `rstest` for parameterized Rust tests, Jest for Node.js tests.

**Test Organization**:
- Unit tests: `#[cfg(test)]` modules in source files
- Integration tests: `tests/` directory and `crates/acceptance/`
- Test naming: `test_<functionality>` pattern

**Running Tests**:
- Rust: `cargo test` or `cargo test --workspace`
- Node.js: `just test-node`
- Integration: `just integration` (uses recorded responses)

**Coverage**: No specific coverage requirements, but comprehensive testing expected for new features.

## Commit & Pull Request Guidelines

The commit-message contract and signing flow are machine-wide — see
`~/.claude/CLAUDE.md`. In short: commit **unsigned** as you go via the `/commit`
skill (`.claude/skills/commit/SKILL.md`); **sign the whole branch once before
opening a PR**. Prefer small, well-scoped commits — release-plz reads them.

Repo-specific rules:
- **Generated code in the same commit** as the change that produced it (run
  `just generate` after proto changes) so reviewers trace generation to output
  in one diff. Stage by name; never `git add -A`.

### bun.lock proxy URLs

Local Bun installs may bake a private npm registry/proxy URL into the resolution
field of `bun.lock`. **Never open a PR with those URLs committed.**

Before opening a PR that touches `bun.lock`:

```bash
bun run strip-lock-proxy
bun run strip-lock-proxy:check
```

The pre-commit hook (`.githooks/pre-commit`) strips proxy URLs when `bun.lock` is
staged, but still run the commands above before opening a PR. Full details:
`.agents/rules/bun-clean-lock-rule.mdc` (also linked from
`.cursor/rules/bun-clean-lock-rule.mdc` for Cursor).

### Quick pre-push check (mimics CI)

```bash
cargo fmt --all --check \
  && cargo clippy --workspace --all-targets --all-features -- -D warnings \
  && cargo nextest run --workspace --all-features --profile ci -E 'not binary(commit_coordinator)' \
  && cargo test --workspace --all-features --doc
```

### Pull Request workflow

1. **Create a feature branch** before any implementation — never work on `main`
   (`git checkout -b feat/<short-description>`).
2. **Create follow-up issues** (`gh issue create`) *before* opening the PR so the
   body can reference them — e.g. deferred migrations, sibling-crate work.
3. **Commit (unsigned) → push → open the PR in one pass** — don't wait on
   signing mid-flow (signatures aren't required to merge). `gh pr create`, target
   `main`:
   - Title: `<type>: <description> (#<issue>)` — reference the closed issue.
   - Body: bullet summary, test-plan checklist, `Closes #N`, follow-up refs, and
     the `This pull request was AI-assisted by Isaac.` line.
4. **Sign once at the end** — surface the combined sign + `--force-with-lease`
   command from `~/.claude` for the user to run (one PIN); can happen any time
   before merge. See the `/commit` skill.

### GitHub Issues workflow

Use the `/create-issue` skill to plan and structure work as GitHub issues (taxonomy, body templates, GraphQL API operations).

Use the `run-issue` agent to implement work from an existing issue (fetches the issue, determines type, orchestrates DAG/wave dispatch).