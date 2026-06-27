# Multi-stage build for the `uc` Unity Catalog CLI/server binary.
#
# Layout:
#   chef    -> installs cargo-chef for dependency-layer caching
#   planner -> computes the dependency recipe from the workspace
#   builder -> cooks deps (cached layer), then compiles the `uc` release binary
#   cleaner -> copies a distroless rootfs into a staging dir (distroless has no
#              shell/rm, so file surgery has to happen in a stage that does)
#   final   -> scratch + the distroless rootfs + the `uc` binary, run as nonroot
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
# Build dependencies first — this is the cached Docker layer that survives
# source-only changes. Scope to the cli package: a bare workspace build unifies
# features across all members, which would pull pyo3 (via python/client) into
# `uc` and force a link against libpython that isn't present in this image.
RUN cargo chef cook --release --recipe-path recipe.json -p olai-uc-cli --bin uc

# Build the application. SQLX_OFFLINE uses the committed `.sqlx` query cache so
# no database is required at build time. `-p olai-uc-cli` keeps feature
# unification scoped to the cli package (see above) so pyo3 stays out.
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release -p olai-uc-cli --bin uc

# Distroless `cc` (nonroot) gives us libgcc/libssl-free glibc + CA certs +
# a nonroot passwd entry, without a shell or package manager.
# gcr.io/distroless/cc-debian12:nonroot
FROM gcr.io/distroless/cc-debian12@sha256:b0ae8e989418b458e0f25489bc3be523718938a2b70864cc0f6a00af1ddbd985 AS base

# busybox:1.37.0 — has a shell + coreutils so we can stage the distroless
# rootfs into a plain directory before copying it into `scratch`.
FROM busybox@sha256:9532d8c39891ca2ecde4d30d7710e01fb739c87a8b9299685c63704296b16028 AS cleaner

COPY --from=base / /clean


FROM scratch

# OCI image labels. The release workflow layers its own via docker/metadata-action,
# but baking them here keeps locally/`just build-docker`-built images labelled too.
LABEL org.opencontainers.image.title="hydrofoil" \
      org.opencontainers.image.description="Unity Catalog server and CLI (uc)" \
      org.opencontainers.image.source="https://github.com/open-lakehouse/mangrove" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.vendor="open-lakehouse"

# Bring in the distroless rootfs (CA certs, glibc, nonroot passwd/group).
COPY --from=cleaner /clean /

# Copy the release binary from the build stage.
COPY --from=builder /app/target/release/uc /usr/local/bin/uc

# The server binds 0.0.0.0:8080 by default (`uc server`).
EXPOSE 8080

# Run unprivileged. The distroless rootfs above provides the `nonroot` (65532)
# user/group entries this references.
USER nonroot

# No HEALTHCHECK: distroless has no shell and `uc` has no self-probe subcommand.
# Orchestrators should health-check the REST API directly, e.g.
#   GET /api/2.1/unity-catalog/catalogs
# (see dev/uc-oss.compose.yaml for a compose-level healthcheck example).

ENTRYPOINT ["/usr/local/bin/uc"]
