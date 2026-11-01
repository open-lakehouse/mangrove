# Contribution Guide

## Getting Started

### Prerequisites

- Rust toolchain ([install instructions](https://www.rust-lang.org/tools/install))
- buf ([install instructions](https://buf.build/docs/installation))
- just ([install instructions](https://just.systems/man/en/))
- bun ([install instructions](https://bun.sh/docs/installation))
- For codegen only: the `trestle` CLI (`brew install open-lakehouse/tap/trestle`) and
  a [`trestle`](https://github.com/open-lakehouse/trestle) source checkout at
  `../trestle` (or `TRESTLE_ROOT`). `cargo check` / `cargo test` need neither.

Enable the repo git hooks once per clone with `bun run setup-hooks` (sets
`core.hooksPath=.githooks`). The hook chains any global hook and guards both
lockfiles. Because `core.hooksPath` is then set, the `pre-commit` framework
(`.pre-commit-config.yaml`: Biome, typos, Ruff, cargo fmt/check) can't install as a
hook alongside it; run `pre-commit run` by hand, or skip `setup-hooks` and use
`pre-commit install` instead (then run the lockfile checks yourself:
`bun run strip-lock-proxy:check`, `bun run cargo-lock:check`).

Behind the Databricks corporate network, point your user-level `~/.npmrc` at the
internal npm mirror (`registry=https://npm-proxy.cloud.databricks.com/`); a direct
`registry.npmjs.org` fetch fails TLS inspection. Bun records the mirror's tarball URLs
in `bun.lock`, which CI runners can't reach, so the committed lock keeps those
resolution fields empty: a root `postinstall` strips them after every `bun install`,
the pre-commit hook strips a staged `bun.lock` as a backstop, and CI fails on any
leftover (`bun run strip-lock-proxy:check`).

### Task runner layout (`just`)

Recipes are organized into [`just` modules](https://just.systems/man/en/modules.html)
by concern. The root `justfile` only declares the modules and a few back-compat
aliases; the recipes live in `just/*.just` (and the promoted sibling justfiles):

| Module        | File                       | Covers                                          |
| ------------- | -------------------------- | ----------------------------------------------- |
| `codegen`     | `just/codegen.just`        | proto / OpenAPI / client generation             |
| `build`       | `just/build.just`          | UI, bindings, wasm engine, Docker, sqlx caches  |
| `test`        | `just/test.just`           | unit / integration / conformance + CI gates     |
| `meta`        | `just/meta.just`           | `fmt`, `fix*`, `lint-node`, `run`                |
| `dev`         | `dev/justfile`             | docker-compose env + UC server / UI dev flows   |
| `postgres`    | `crates/postgres/justfile` | sharing migrations helpers                       |
| `node_client` | `node/client/justfile`     | node client external-type generation            |

Invoke recipes namespaced — `just codegen generate`, `just build ui-build`,
`just test test-node`, `just dev rest`, `just meta fmt`. List a module's recipes
with `just --list <module>` (e.g. `just --list codegen`) or the whole tree with
`just --list`. A handful of names CI and this guide depend on (`just generate`,
`just sync-server-openapi-check`, …) are kept working flat via aliases in the root
`justfile`; new usage should prefer the namespaced form.

## Generated Code

We heavily rely on code generation to ensure consistency with the API spec and to reduce the maintenance burden.
The most important components involved in our code generation are:

- The `protobuf` definitions which define the API surface.
- [`buf.gen.yaml`](buf.gen.yaml) which defines the code we generate using `buf`
- the external [`trestle`](https://github.com/open-lakehouse/trestle) codegen
  tool, which holds the custom generation logic. It must be checked out as a
  sibling directory (`../trestle`) for `just generate*` to work.

The Unity Catalog API is specified as a REST API, but we maintain API definitions in
protobuf for more flexible code generation and better maintainability. To map protobuf
messages/services to REST endpoints, we annotate definitions with
[`google.api.http`](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto)
and [`gnostic`](https://github.com/google/gnostic) options.

These annotations are used by the `buf` compiler to generate OpenAPI specifications
and by our custom code to provide boilerplate server/client implementations.

Run the complete generation sequence:

```sh
bun install
just generate
```

### Adding new resources

To add a new resource/API surface, follow these steps:

1. **Define protobuf schema**: Create the resource in `proto/unitycatalog/<resource>/v1/`
   - Define messages (e.g., `Volume`, `CreateVolumeRequest`)
   - Define service with RPC methods
   - Annotate with `google.api.http` and `gnostic.openapi.v3.operation`

2. **Generate base code**: Run `just generate-proto` to generate common models

3. **Wire up the models**: mount the new package in
   `crates/common/src/models/generated.rs` (and `generated_extractors.rs`) behind a
   feature of the same name, and add that feature to `all-resources` in
   `crates/common/Cargo.toml`. The drift test in `generated.rs` fails until you do.

4. **Generate clients**: Run `just generate-code` for the handler traits
   (`crates/handlers`), client and server boilerplate

5. **Hand-written client ergonomics** (only where codegen falls short): add them next
   to the existing helpers in `crates/client/src/` and export them from `lib.rs`.

6. **Python bindings** are generated: `just generate-code` emits the extension
   module (`python/client/src/codegen`), with the new API's model classes and
   resource client in a submodule `unitycatalog_client._client.<api>`, and the enums
   in `_enums.py`. Then:
   - Run `just codegen generate-py-stubs` to rewrite the stubs (`_client/*.pyi`).
   - For a new proto package, add its `from ._client.<api> import *` line to
     `python/client/python/unitycatalog_client/__init__.py`.

### Hand-written PyO3 helpers

A few Python bindings are not proto-derived (e.g. `parse_uc_url`, ergonomic
`temporary_*_credential` methods that resolve name → UUID before calling the
generated RPC). They live in `python/client/src/{client,reference}.rs`. The
extension module is generated, so:

- **Export the item through `trestle.yaml`**: add its Rust path to
  `clients.python.options.exports`, and trestle mounts it in the generated
  `#[pymodule]`. Give a class `module = "unitycatalog_client._client"`.
- **Stubs come from the compiled module**: `just codegen generate-py-stubs` reads
  PyO3's introspection data, so hand-written items get stubs like generated ones.
  **Never hand-edit `_client/*.pyi`.**
- **Re-export public classes at the package root** with the PEP 484 form
  `from ._client import Foo as Foo` in
  `python/client/python/unitycatalog_client/__init__.py`. Keep internal helpers
  (e.g. `parse_uc_url`) out of the root; consumers import them from `..._client`.
- **Client errors**: `python/client/src/error.rs` implements the generated
  `ApiErrorAnswer` for `unitycatalog_client::Error`, which picks the exception
  class (by HTTP status) and its `code`/`message`.
- **For proto-shaped surface** (a regular `Get/Update/Delete/Create/List` RPC, or
  a `Custom(Post|Patch)` RPC the Python emitter renders), prefer extending the
  proto so trestle generates everything end-to-end.

## Service architecture: server binary, client CLI, bundled UI

This repo follows a **shared service/CLI/UI layout** that the sibling
[`headwaters`](https://github.com/open-lakehouse/headwaters) repo also uses. When adding a
new deployable service or CLI, keep to this contract so the two stay aligned:

- **Two binaries.**
  - **`uc-server`** (crate `olai-uc-server`, `crates/server`) is the deployable service. It
    has three subcommands:
    - `serve` — run the REST service. It does **not** migrate a durable
      backend; run `migrate` first. *Exception:* an ephemeral `:memory:` SQLite backend is
      auto-migrated at startup (a separate `migrate` process can't reach a fresh in-process
      DB), so a config-less `uc-server serve` works out of the box.
    - `migrate` — apply pending migrations and exit. The only schema-mutating path, kept
      off the `serve` hot path so concurrent replicas don't race.
    - `healthcheck` — probe `/health` with a blocking client and map the result to a
      process exit code. This is what the distroless image's Docker `HEALTHCHECK` runs
      (distroless has no shell/curl).
  - **`uc`** (crate `olai-uc-cli`, `crates/cli`) is a thin HTTP **client** for a running
    server (`client`, `explore`). It does not depend on the server crate. (Mirrors
    headwaters' `hw`.)
- **Operational endpoints.** The server always serves `/health` (body `OK`) and `/version`
  (crate version), regardless of backend or routing; they sit outside the auth layer so
  probes work unauthenticated.
- **Layered config.** `serve` loads a YAML config file (also `UC_SERVER_CONFIG`), then
  overlays CLI flags (`--host`/`--port`/`--no-ui`), CLI winning. UI serving is a
  `UiConfig { base_path, serve }` sub-struct; `--no-ui` can only *disable* serving.
- **Bundled UI.** The server serves a built single-page app from `./web` on disk via
  `tower-http::ServeDir` (SPA `index.html` fallback), gated by `ui.serve` (default on) /
  `--no-ui`, and optionally mounted under `ui.base_path` for a gateway sub-path. The SPA is
  the `node/app` package (`@open-lakehouse/uc-app`), a thin Vite app consuming the shared,
  reusable `@open-lakehouse/*` component packages (`unity-catalog`, `ui-kit`, `data-grid`,
  `unity-catalog-client`). Build it with `just ui-build`; run the server serving both API
  and UI on one origin with `just rest-ui`. The Docker image bakes `node/app/dist` into
  `./web`. *(Publishing the shared component packages to npm is deferred while names
  settle — they stay workspace-internal for now.)*
- **Release.** The server is a deployable (`git_only`, Docker), not a crates.io library;
  its `node/` UI releases it through the `uc-app` placeholder crate. See **Releases** below.

## Releases

Releases are driven by [release-plz](https://release-plz.dev) from
[Conventional Commits](https://www.conventionalcommits.org). You never bump versions or
write changelogs by hand. The PR title is the squash-merge commit, so CI lints it against
the convention (`.github/workflows/pr-title.yml`); the commit type drives the semver bump.

**Provisional crate names.** The library/CLI crates are published to crates.io under
provisional `olai-uc-*` names (e.g. `olai-uc-common`, `olai-uc-cli`) while the public
design settles. To keep this rename source-free, each crate's package `name` is
`olai-uc-*` but the **in-workspace dependency key stays `unitycatalog(-)*`** via Cargo's
`package =` alias (the same trick used for `delta_kernel`/`buoyant_kernel`). So source
keeps `use unitycatalog_common::…`; only `Cargo.toml` carries the published name.

**Each crate versions independently.** release-plz bumps a crate from the commits that
touch it (and bumps its dependents automatically), so e.g. `olai-uc-object-store` and
`olai-uc-datafusion` can release on their own cadence. Config: `release-plz.toml` (the
changelog/git-cliff config is embedded there — there is no separate `cliff.toml`).

PRs are squash-merged, so the **PR title is the only commit release-plz sees**. Use a
`(scope)` matching the affected crate's short name — `common`, `client`,
`object-store`, `postgres`, `sqlite`, `datafusion`, `server`, `cli` — so the bump and
changelog land on the right crate. Commits inside a PR only matter for review. When a
change spans several crates or change types that each deserve their own changelog
entry, split it into a stack of PRs (one `type(scope)` per layer; see `AGENTS.md`)
rather than one mixed PR.

**Cross-repo (trestle) dependencies.** The trestle crates (`olai-http`, `olai-store`,
`olai-storage-credentials`, `olai-vended-store`, `olai-credential-vending`) are declared as plain
published `version` deps in the root `Cargo.toml` (e.g. `olai-http = { version = "0.0.3" }`).
They are **not** `path` deps: a committed `path` must physically exist for *every* cargo
command, but CI has no `../trestle` checkout, so a committed path aborts CI; and a `git`
source would block `cargo publish`. CI and `cargo publish` resolve the version from
crates.io natively.

To build against **local trestle source** — e.g. before a change is published, or while
the crates.io proxy's 7-day age filter still hides a freshly published version — run
`just configure-trestle-deps`. It writes a gitignored `.cargo/config.toml` that patches
every trestle crate the locks use to the local checkout (found next to the main mangrove
checkout, so worktrees work too, or via `TRESTLE_ROOT`):

```toml
# >>> crates-proxy:trestle (written by trestle/scripts/crates-proxy/patch.sh; never commit)
patch.crates-io.olai-http = { path = "/path/to/trestle/crates/olai-http" }
# ...
# <<< crates-proxy:trestle
```

Cargo also reads it when building from `wasm/`. A build under the patch rewrites the
`Cargo.lock` files without the crates' registry `source`/`checksum`. Before committing,
run `just dev lock-registry`: it restores those entries, computing each checksum from the
crate's tarball on the `static.crates.io` CDN, which the proxy doesn't filter. The
pre-commit hook (`just dev check-lock`) rejects a lock that still points at the patch.
Do **not** commit a patch (CI has no `../trestle`, so it would fail). Remove it with
`just dev unpatch-deps trestle` once the proxy catches up. When you need an unreleased
trestle change, commit a git pin (`{ version, git, rev }`) or release trestle first.

None of this is trestle-specific. The same delay hits every crate a sibling repo
publishes. `just dev patch-deps <upstream>` patches any of them from its checkout, and
patching several upstreams for chained releases works too. mangrove's own releases put
its consumers (distributary, hydrofoil) in the same position, and they use the same
tooling. The scripts behind these recipes live in trestle; see its
[`docs/crates-proxy.md`](https://github.com/open-lakehouse/trestle/blob/main/docs/crates-proxy.md).

**How a release happens:**

1. Merge PRs to `main` with conventional-commit titles (`feat:`, `fix:`, `feat(scope)!:`
   for breaking changes).
2. release-plz opens/updates a **Release PR** that bumps the affected crates' versions
   and updates their changelogs. Review it like any PR.
3. **Merging the Release PR** publishes: release-plz tags each changed crate
   (`<crate>-v<version>`), creates its GitHub Release, and publishes the publishable crates
   to crates.io via OIDC trusted publishing. When a release includes `olai-uc-server`,
   release-plz.yml also drives the `mangrove` container build off release-plz's `releases`
   output (see below). `olai-uc-server` is **paused** — not currently published or built.

**Tags / artifacts:**

| Tag                    | Builds & attaches                                  | Workflow                          |
|------------------------|----------------------------------------------------|-----------------------------------|
| every `olai-uc-*-v*`   | GitHub Release (changelog); crates.io publish for publishable crates | release-plz.yml   |
| `olai-uc-server-v*`    | + `ghcr.io/open-lakehouse/mangrove` image (once the server is un-paused) | release-plz.yml → docker-release.yml |
| `unity-catalog-node-v*` / `unity-catalog-client-v*` | npm package on GitHub Packages + attested tarball on the Release | release-plz.yml → js-release.yaml |

**The server is a deployable — currently paused.** `olai-uc-server` is `git_only = true` in
`release-plz.toml` (and `publish = false` in its `Cargo.toml`): release-plz versions,
changelogs, and tags it from git but does not `cargo publish` it. The crate's own dependency
graph is git-free (PR #89), so it is *eligible* to publish to crates.io (library + the
`uc-server` binary behind the `bin` feature), but it is held back until its sibling path deps
(`olai-uc-common`/`-client`/`-delta-api`) are re-released to crates.io with the current buffa
API — a verifying `cargo publish` of the server otherwise builds against their stale pre-buffa
`0.0.2` and fails. The Docker gate in release-plz.yml reads release-plz's `releases` JSON. (Since
release-plz action v0.5.142 that JSON also lists released `git_only` crates; the npm
packages rely on it.) Flipping the server to `publish = true` (after the siblings are live) puts it back
in `releases` and re-activates the Docker build automatically — no workflow edit needed; the
image then builds only on an actual server release or a manual `workflow_dispatch` rebuild
(there is no per-push/edge build). The server's bundled web UI lives in `node/` — outside the
crate's packaged fileset. A placeholder crate at `node/Cargo.toml` (`uc-app`, `git_only`,
never published) owns every tracked file under `node/`, and the server lists it (with
`unity-catalog-client`) in `changelog_include` in `release-plz.toml`. A UI change therefore
releases `uc-app` and the server (at most a patch bump), and the server tag rebuilds the
image. Nothing needs regenerating by hand.

The Python binding (`python/client`) is `publish = false` and released by hand (see
`python-wheels.yml`); so are `unitycatalog-acceptance` and the doc `examples`.

**npm packages (GitHub Packages).** The publishable npm packages are versioned by
release-plz through crates, the same way trestle releases `trestle-diagram`:

| npm package | Crate | Manifest |
|---|---|---|
| `@open-lakehouse/unity-catalog-node` | `unity-catalog-node` | `node/client/Cargo.toml` (the NAPI extension itself) |
| `@open-lakehouse/unity-catalog-client` | `unity-catalog-client` | `node/unity-catalog-client/Cargo.toml` (placeholder, no Rust) |

Both are `git_only = true` + `publish = false`. The crate `version` is the npm version
(`package.json` stays `0.0.0` in git), and each crate's `include` list is exactly the
files that feed its npm package, so release-plz bumps it when those change.
`unity-catalog-node` also bumps whenever `olai-uc-client` / `olai-uc-common` release,
through its path dependencies. When a run releases one of them, release-plz.yml's `npm`
job calls `js-release.yaml`. That job stamps the version into `package.json`, builds
(NAPI: one binary per target, plus the per-platform packages), publishes to
`npm.pkg.github.com`, attests the tarballs, and attaches them to the GitHub Release. Nothing
is published unless the `JS_PUBLISH_ENABLED` repo variable is `"true"`. A release tagged
while it was off can be published later via `workflow_dispatch` on `js-release.yaml`
with its tag. No npmjs publishing yet.

- When a published package gains source files outside its crate's `include` list, add
  them, or changes to them won't release the package
  (`cargo package -p <crate> --list --allow-dirty` shows the set).
- A new publishable npm package needs a crate (placeholder if it has no Rust), a
  `[[package]]` entry in `release-plz.toml`, its dir in `js-release.yaml`'s crate → dir
  map, and the crate name in `release-plz.yml`'s npm detect step. Workspace packages
  depend on it with `workspace:*`.
- The UI packages stay `private: true`; don't add npmjs publishing or un-private one
  without a maintainer asking for it.

**First-publish bootstrap.** crates.io OIDC trusted publishing cannot create a crate name
that has never existed. Each `olai-uc-*` crate therefore needs a one-time token bootstrap
before release-plz can publish it via OIDC. Until then it carries `release = false` in
`release-plz.toml` (release-plz still versions/changelogs/tags it, just doesn't publish).
The bootstrap runs from `.github/workflows/bootstrap-publish.yml`. For each crate:
token-publish it, register its crates.io Trusted Publisher (repo `open-lakehouse/mangrove`,
workflow `release-plz.yml`, env `release`), then remove its `release = false`.
`olai-uc-server` is **not** in this chain today — it is `git_only` (Docker only) until its
sibling deps are re-released; when it flips to a published crate it will need this one-time
bootstrap. Once every publishable crate is live, delete the bootstrap workflow and revoke the
`CARGO_REGISTRY_TOKEN` secret. Any *new* publishable crate added later needs the same
one-time bootstrap (re-add the workflow if so). The Open Sharing crates
(`olai-sharing-*`) moved to their own repository,
[distributary](https://github.com/open-lakehouse/distributary). Not published to crates.io at
all: `olai-uc-datafusion` (standing git-dep block via the `deltalake-core` git rev) and
`olai-uc-cli` (held out for now).

**Notes:**

- Versions live committed in each `Cargo.toml`; release-plz writes them via the Release PR.
  Never edit a version manually and never use a placeholder — artifacts build from the
  committed version at a real commit SHA, which the provenance attestations bind to.
- crates.io publishing uses OIDC (`id-token: write`); no `CARGO_REGISTRY_TOKEN` is needed
  in steady state (only the temporary bootstrap workflow uses one).
- An optional `RELEASE_PLZ_TOKEN` (PAT/App token) lets the Release PR run CI under a
  non-`GITHUB_TOKEN` identity; the workflow falls back to `GITHUB_TOKEN` when it is unset.

## AI-assisted contributions

AI-assisted changes are welcome. Understand the diff before submitting it, match
the surrounding style, and don't include code you can't explain. Agent instructions
live in [`AGENTS.md`](../AGENTS.md) (imported by `CLAUDE.md`).
