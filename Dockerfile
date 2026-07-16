# syntax=docker/dockerfile:1
#
# Build for the `uc-server` binary: the deployable Unity Catalog server, with the
# bundled web UI baked in. The build context is the repo root — cargo-chef caches
# the dependency graph as a separate layer so source edits don't trigger a full
# dependency rebuild, and a separate node stage builds the SPA so the node
# toolchain never reaches the runtime image.
#
# Base images are pinned by digest for reproducible, tamper-evident builds.
# Refresh a digest with: docker buildx imagetools inspect <image:tag>

# rust:1.96-bookworm
FROM rust@sha256:19817ead3289c8c631c73df281e18b59b172f6a31f4f563290f69cddd06c30e9 AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build the bundled single-page web UI (node/app + the @open-lakehouse/* workspace
# packages it consumes) into static assets. The server serves these from ./web at
# runtime. Kept as its own stage so the (large) node toolchain never reaches the
# runtime image and the bun install layer caches independently of the Rust build.
FROM oven/bun:1.3.14-debian AS ui
WORKDIR /ui
# Lockfile + manifests first for a cacheable `bun install` layer. The build needs the
# whole bun workspace (root manifest + every node/* package the app imports).
COPY package.json bun.lock ./
COPY node/ ./node/
# The root manifest also lists the `docs` and `examples/typescript` workspaces, which
# the server image never builds. With `--frozen-lockfile`, bun still resolves EVERY
# workspace glob against the on-disk tree, so their manifests must exist or install
# fails with `Workspace not found`. Copy just the package.json (not the source): bun
# resolves the workspace but never fetches its deps, since nothing the app builds
# depends on them — the docs toolchain (astro/sharp) stays out of the image. Editing
# the lockfile to drop them instead would trip the frozen-lockfile check.
COPY docs/package.json ./docs/
COPY examples/typescript/package.json ./examples/typescript/
# The committed bun.lock pins each tarball URL to whatever registry it was generated
# against — for us some entries may point at an internal mirror
# (npm-proxy.cloud.databricks.com) that CI and other external builders can't reach.
# Re-point the host to the target registry here; the integrity hashes are unchanged
# (identical tarball content on any mirror), so the lockfile's guarantees still hold.
# Defaults to the public registry so CI works out of the box; override NPM_REGISTRY
# (and NPM_REGISTRY_FROM, the host to replace) to build behind a different mirror.
ARG NPM_REGISTRY=https://registry.npmjs.org
ARG NPM_REGISTRY_FROM=https://npm-proxy.cloud.databricks.com
RUN sed -i "s#${NPM_REGISTRY_FROM}/#${NPM_REGISTRY}/#g" bun.lock \
    && bun install --frozen-lockfile
# Build the app → node/app/dist. Vite `base: "./"` makes the asset URLs relative
# so one image works under any server base-path without a rebuild.
RUN bun run --filter @open-lakehouse/uc-app build

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies first — this is the cached Docker layer that survives
# source-only changes. Scope to the server package + `bin` feature so a bare
# workspace build doesn't unify features across all members (which would pull
# pyo3 via python/client and force a libpython link this image lacks).
RUN cargo chef cook --release --recipe-path recipe.json \
    -p olai-uc-server --features bin --bin uc-server
# Build the application. SQLX_OFFLINE uses the committed `.sqlx` query cache so no
# database is required at build time.
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release -p olai-uc-server --features bin --bin uc-server

# Minimal runtime: distroless cc (glibc + openssl + CA certs) for the
# dynamically-linked binary, nonroot. No shell/package manager, so the
# healthcheck below can't shell out to curl/wget — it runs the binary's own
# `healthcheck` subcommand instead.
# gcr.io/distroless/cc-debian12:nonroot
FROM gcr.io/distroless/cc-debian12@sha256:b0ae8e989418b458e0f25489bc3be523718938a2b70864cc0f6a00af1ddbd985 AS runtime

LABEL org.opencontainers.image.title="mangrove" \
      org.opencontainers.image.description="Mangrove — a lakehouse catalog server (Unity Catalog + Delta Sharing APIs) with bundled web UI" \
      org.opencontainers.image.source="https://github.com/open-lakehouse/mangrove" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.vendor="open-lakehouse"

COPY --from=builder /app/target/release/uc-server /usr/local/bin/uc-server
# The server serves the bundled SPA from `./web` relative to its working
# directory (see UI_DIR in crates/server/src/run.rs), so run from /app and drop
# the bundle there. Absent the bundle the SPA routes just 404 — the API still serves.
WORKDIR /app
COPY --from=ui /ui/node/app/dist ./web

# The server binds 0.0.0.0:8080 by default (`serve`).
EXPOSE 8080

# Self-probe: the binary GETs its own /health and exits 0/1. Exec-form (JSON
# array) is REQUIRED — distroless has no shell for the string form. The probe
# reads the same config/env the server does, so it targets the right port.
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD ["/usr/local/bin/uc-server", "healthcheck"]

USER nonroot

# ENTRYPOINT is just the binary; CMD supplies the default subcommand. Split this
# way so a CMD-less `docker run` starts the server (`serve`) while
# `docker run … migrate` / `… healthcheck` (or a Compose `command:`) *replaces*
# CMD to pick the subcommand. NOTE: `serve` does NOT migrate a durable backend —
# run `uc-server migrate` once before the first `serve` against a new database.
ENTRYPOINT ["/usr/local/bin/uc-server"]
CMD ["serve"]
