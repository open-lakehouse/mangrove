# Contribution Guide

## Getting Started

### Prerequisites

- Rust toolchain ([install instructions](https://www.rust-lang.org/tools/install))
- buf ([install instructions](https://buf.build/docs/installation))
- just ([install instructions](https://just.systems/man/en/))
- bun ([install instructions](https://bun.sh/docs/installation))

Behind the Databricks corporate network, point your user-level `~/.npmrc` at the
internal npm mirror (`registry=https://npm-proxy.cloud.databricks.com/`). `bun.lock`
stores the mirror host in each tarball URL, so installs go through the proxy
instead of hitting `registry.npmjs.org` directly (which fails TLS inspection).
CI and Docker builds rewrite those URLs to the public registry at install time.

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

3. **Update exports**: Add new types to `unitycatalog_common::models` module exports

4. **Generate clients**: Run `just generate-code` for server/client boilerplate

5. **Implement high-level client**:
   - Create `crates/client/src/<resource>.rs` with ergonomic methods
   - Add to `lib.rs` exports and main client struct
   - Add streaming support for list operations

6. **Add Python bindings**:
   - Import new types in `python/client/src/lib.rs`; add the client wrapper in
     `python/client/src/client.rs`.
   - Re-run `just generate-code` (regenerates `_client.pyi`), then re-export any
     new pyclass at the package root — see **Hand-written PyO3 helpers** below
     for the `from ._client import Foo as Foo` convention.

### Hand-written PyO3 helpers

A few Python bindings are not proto-derived (e.g. `parse_uc_url`, ergonomic
`temporary_*_credential` methods that resolve name → UUID before calling the
generated RPC). They live in `python/client/src/{client,reference}.rs`; type
checkers can't read their attributes off the compiled `.so` and trestle codegen
doesn't see them. **Never hand-edit `_client.pyi`** (it is fully regenerated).
Instead:

- **Declare the symbol in `python/client/_client_supplement.pyi`.** `just
  generate-code` appends this fragment to the codegen-emitted `_client.pyi`, so
  the merged stub describes the full `_client` runtime surface. The supplement
  lives outside the package dir so type checkers don't validate it standalone.
- **Re-export from the package root** via the PEP 484 form `from ._client import
  Foo as Foo` in `python/client/python/unitycatalog_client/__init__.py` — the
  same idiom used for codegen-derived types. Do this whenever you
  `m.add_class::<Foo>()` (or register a new exception / free function) in
  `python/client/src/lib.rs`. Keep internal helpers (e.g. `parse_uc_url`) out of
  the root re-export list; consumers import them from `..._client` directly.
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
    - `serve` — run the REST/Delta-Sharing service. It does **not** migrate a durable
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
  its `node/` UI is tied to the crate via `crates/server/ui.lock`. See **Releases** below.

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

Use a `(scope)` matching the affected crate's short name — `common`, `client`,
`sharing-client`, `object-store`, `postgres`, `sqlite`, `datafusion`, `server`, `cli` —
so the bump and changelog land on the right crate. Prefer several focused commits over one
mixed commit; keep generated output in the same commit as its source.

**Cross-repo (trestle) dependencies.** `olai-http` / `olai-store` are declared as plain
published `version` deps in the root `Cargo.toml` (e.g. `olai-http = { version = "0.0.3" }`).
They are **not** `path` deps: a committed `path` must physically exist for *every* cargo
command, but CI has no `../trestle` checkout, so a committed path aborts CI; and a `git`
source would block `cargo publish`. CI and `cargo publish` resolve the version from
crates.io natively.

To build against **local trestle source** — e.g. before a change is published, or while
the crates.io proxy's 7-day age filter still hides a freshly published version — add a
**local, uncommitted** `[patch.crates-io]` to the root `Cargo.toml`:

```toml
[patch.crates-io]
olai-http  = { path = "../trestle/crates/olai-http", version = "X.Y.Z" }
olai-store = { path = "../trestle/crates/olai-store", version = "X.Y.Z" }
```

Do **not** commit that block (CI has no `../trestle`, so it would fail). When you need a
trestle change permanently, release trestle to crates.io *first*, then bump the `version`
here.

**How a release happens:**

1. Merge PRs to `main` with conventional-commit titles (`feat:`, `fix:`, `feat(scope)!:`
   for breaking changes).
2. release-plz opens/updates a **Release PR** that bumps the affected crates' versions
   and updates their changelogs. Review it like any PR.
3. **Merging the Release PR** publishes: release-plz tags each changed crate
   (`<crate>-v<version>`), creates its GitHub Release, and publishes the publishable crates
   to crates.io via OIDC trusted publishing. When a release includes `olai-uc-server`,
   release-plz.yml additionally drives the `mangrove` container build off release-plz's own
   `releases` output (see below).

**Tags / artifacts:**

| Tag                    | Builds & attaches                                  | Workflow                          |
|------------------------|----------------------------------------------------|-----------------------------------|
| every `olai-uc-*-v*`   | GitHub Release (changelog); crates.io publish for publishable crates | release-plz.yml   |
| `olai-uc-server-v*`    | + `ghcr.io/open-lakehouse/mangrove` image (in addition to the crates.io publish) | release-plz.yml → docker-release.yml |

**The server is both a published crate and a Docker image.** `olai-uc-server` publishes to
crates.io like any other crate (library + the `uc-server` binary, the latter behind the
`bin` feature — `cargo install olai-uc-server --features bin`), *and* ships as the
`ghcr.io/open-lakehouse/mangrove` image. Because it is a normal published crate, release-plz
lists it in the `release` command's `releases` JSON; release-plz.yml reads that JSON and, when
the server was released this run, calls `docker-release.yml` with the version + tag. (This
replaced an older bespoke tag-creation shell step that was only needed while the server was
`git_only`.) The Docker image builds **only** on an actual server release or a manual
`workflow_dispatch` rebuild — there is no per-push/edge build. The server's bundled web UI
lives in `node/` — outside the crate's packaged fileset — so a UI change is tied to the crate
via `crates/server/ui.lock`; after editing anything under `node/`, run `just ui-fingerprint`
and commit the updated lock (CI's `ui-fingerprint` job enforces this).

The Python (`python/client`) and Node (`node/client`) bindings are `publish = false` for
now (held off); so are `unitycatalog-acceptance` and the doc `examples`.

**First-publish bootstrap.** crates.io OIDC trusted publishing cannot create a crate name
that has never existed. Each `olai-uc-*` crate therefore needs a one-time token bootstrap
before release-plz can publish it via OIDC. Until then it carries `release = false` in
`release-plz.toml` (release-plz still versions/changelogs/tags it, just doesn't publish).
The bootstrap runs from `.github/workflows/bootstrap-publish.yml`. For each crate:
token-publish it, register its crates.io Trusted Publisher (repo `open-lakehouse/mangrove`,
workflow `release-plz.yml`, env `release`), then remove its `release = false`.
`olai-uc-server` is the **last** crate that needs this bootstrap (it recently became a
published crate); once it is live, delete the bootstrap workflow and revoke the
`CARGO_REGISTRY_TOKEN` secret. Any *new* publishable crate added later needs the same
one-time bootstrap (re-add the workflow if so). Not published to crates.io at all:
`olai-uc-sharing-client` / `olai-uc-sharing-api` / `olai-uc-datafusion` (standing git-dep
block via the `delta_kernel`/`deltalake-core` git revs) and `olai-uc-cli` (held out for now).

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
the surrounding style, and don't include code you can't explain. Every commit
carries the `AI-assisted-by: Isaac` trailer and PR bodies end with the
attribution line — both are defined in `~/.claude/CLAUDE.md`. The commit/sign
mechanics live in the `/commit` skill (`.claude/skills/commit/SKILL.md`).
