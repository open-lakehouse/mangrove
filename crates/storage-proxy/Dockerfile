# syntax=docker/dockerfile:1
#
# Build for the standalone `storage-proxy` binary: the same-origin Unity Catalog
# storage byte-proxy, deployed on its own (no database, no bundled UI). The build
# context is the repo root — cargo-chef caches the dependency graph as a separate
# layer so source edits don't trigger a full dependency rebuild.
#
# Unlike the server image (repo-root `Dockerfile`) there is no node/UI stage and
# no SQLX offline cache: the proxy serves no SPA and has no database.
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

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies first — the cached Docker layer that survives source-only
# changes. Scope to the storage-proxy package + `bin` feature so a bare workspace
# build doesn't unify features across all members (which would pull pyo3 via
# python/client and force a libpython link this image lacks).
RUN cargo chef cook --release --recipe-path recipe.json \
    -p olai-uc-storage-proxy --features bin --bin storage-proxy
COPY . .
RUN cargo build --release -p olai-uc-storage-proxy --features bin --bin storage-proxy

# Minimal runtime: distroless cc (glibc + openssl + CA certs) for the
# dynamically-linked binary, nonroot.
# gcr.io/distroless/cc-debian12:nonroot
FROM gcr.io/distroless/cc-debian12@sha256:b0ae8e989418b458e0f25489bc3be523718938a2b70864cc0f6a00af1ddbd985 AS runtime

LABEL org.opencontainers.image.title="storage-proxy" \
      org.opencontainers.image.description="Unity Catalog storage byte-proxy — same-origin, streaming reads/writes over governed securables" \
      org.opencontainers.image.source="https://github.com/open-lakehouse/mangrove" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.vendor="open-lakehouse"

COPY --from=builder /app/target/release/storage-proxy /usr/local/bin/storage-proxy
WORKDIR /app

# The proxy binds 0.0.0.0:8080 by default (`serve`).
EXPOSE 8080

# Self-probe: the binary GETs its own /health and exits 0/1. Exec-form (JSON
# array) is REQUIRED — distroless has no shell for the string form. With no
# config file the probe targets the default bind address (127.0.0.1:8080).
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD ["/usr/local/bin/storage-proxy", "healthcheck"]

USER nonroot

# ENTRYPOINT is just the binary; CMD supplies the default subcommand. Split this
# way so a CMD-less `docker run` starts the proxy (`serve`) while
# `docker run … healthcheck` (or a Compose `command:`) *replaces* CMD to pick the
# subcommand.
ENTRYPOINT ["/usr/local/bin/storage-proxy"]
CMD ["serve"]
