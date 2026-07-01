mod dev 'dev/justfile'

set dotenv-load

# Show available commands
_default:
    @just --list --justfile {{ justfile() }}

run *args:
    cargo run --bin uc {{ args }}

# main code generation command. This will run all generation for unity types.
[group('codegen')]
generate: generate-proto generate-code generate-code-sharing fix

# run all code generation for unitycatalog types.
#
# Generation of external types (google.api / gnostic extensions) now lives in
# the `../trestle` codegen repo (`olai-codegen`); see `generate-proto-gen-fixtures`.
[group('codegen')]
generate-full: generate-proto generate-code generate-code-sharing fix

# run code generation for proto files.
[group('codegen')]
generate-proto:
    buf generate proto/unitycatalog
    just generate-openapi
    buf generate proto/sharing --template {{ justfile_directory() }}/buf.gen.sharing.yaml
    just generate-openapi-sharing

# Generate the Open Sharing OpenAPI spec from proto/sharing (gnostic) and merge
# in the hand-maintained NDJSON query paths.
[group('codegen')]
generate-openapi-sharing:
    mkdir -p {{ justfile_directory() }}/openapi/sharing-gen
    buf generate proto/sharing --template {{ justfile_directory() }}/buf.gen.sharing-openapi.yaml
    uv run --with pyyaml python3 dev/scripts/merge_sharing_openapi.py \
      openapi/sharing-gen/openapi.yaml \
      openapi/sharing-query-paths.yaml \
      openapi/sharing.yaml
    rm -rf {{ justfile_directory() }}/openapi/sharing-gen
    npx -y @redocly/cli bundle openapi/sharing.yaml > /dev/null

# Update the generated openapi spec with validation extracted from generated jsonschema.
[group('codegen')]
generate-openapi:
    buf generate --template '{"version":"v2","plugins":[{"remote":"buf.build/bufbuild/protoschema-jsonschema:v0.6.0","opt": ["target=proto-strict-bundle"], "out":"openapi/jsonschema"}]}' proto
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- enrich-openapi \
      --jsonschema-dir openapi/jsonschema \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm -f {{ justfile_directory() }}/descriptors.bin
    rm -rf openapi/jsonschema
    npx -y @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    npm run openapi

# generate rest server and client code with build crate.
[group('codegen')]
generate-code:
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- generate --config trestle.yaml \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm {{ justfile_directory() }}/descriptors.bin
    just fmt
    mv python/client/src/codegen/client.pyi python/client/python/unitycatalog_client/_client.pyi
    # Splice in the hand-written PyO3 surface (exceptions, free functions,
    # and the hand-written `#[pymethods]` on `TemporaryCredentialClient`).
    # The codegen-emitted empty `class TemporaryCredentialClient: ...`
    # placeholder is stripped first so the supplement can replace it with
    # the fully-typed declaration. The supplement lives outside the
    # package directory so type-checkers do not validate it standalone
    # (it is a fragment, not a complete stub). Source-of-truth:
    # `python/client/_client_supplement.pyi`.
    grep -v '^class TemporaryCredentialClient: \.\.\.$' \
      python/client/python/unitycatalog_client/_client.pyi \
      > python/client/python/unitycatalog_client/_client.pyi.tmp
    mv python/client/python/unitycatalog_client/_client.pyi.tmp \
      python/client/python/unitycatalog_client/_client.pyi
    cat python/client/_client_supplement.pyi \
      >> python/client/python/unitycatalog_client/_client.pyi

# generate sharing (Open Sharing) server/client/extractor code from proto/sharing.
#
# The sharing surface lives in its own crate (`unitycatalog-sharing-client` for
# models + co-located extractors + client, `unitycatalog-server` for handler
# traits/routes), so it has its own trestle config (`trestle.sharing.yaml`)
# separate from the resource-oriented Unity Catalog pipeline in `generate-code`.
# The NDJSON table
# query RPCs are intentionally excluded from the proto service and implemented by
# hand (see `crates/sharing-client/src/query_extractors.rs`).
[group('codegen')]
generate-code-sharing:
    buf build --output {{ justfile_directory() }}/sharing-descriptors.bin proto/sharing
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- generate --config trestle.sharing.yaml \
      --descriptors {{ justfile_directory() }}/sharing-descriptors.bin
    rm {{ justfile_directory() }}/sharing-descriptors.bin
    just fmt

# CURRENTLY not used, but we may need it again come validation ...
[group('codegen')]
generate-common-ext:
    just crates/common/generate

# generate types for node client. these are all slow changing external types
[group('codegen')]
generate-node:
    just node/client/generate

# Regenerate proto-gen test fixture descriptors from proto/ source files.
[group('codegen')]
generate-proto-gen-fixtures:
    buf dep update ../trestle/crates/trestle-codegen/proto
    buf build --output {{ justfile_directory() }}/../trestle/crates/olai-codegen/proto/example.bin \
      ../trestle/crates/olai-codegen/proto/

# run the development REST server (ephemeral in-memory SQLite; auto-migrated)
[group('dev')]
rest:
    @RUST_LOG=INFO cargo run -p olai-uc-server --features bin --bin uc-server -- serve

# run the server against Postgres: migrate first (durable backends are not
# auto-migrated by `serve`), then serve. Needs a running Postgres + a config
# file (dev/config.yaml) selecting the postgres backend.
[group('dev')]
rest-db config="dev/config.yaml":
    cargo run -p olai-uc-server --features bin --bin uc-server -- migrate -c {{ config }}
    RUST_LOG=INFO cargo run -p olai-uc-server --features bin --bin uc-server -- serve -c {{ config }}

# build the bundled web UI (node/app -> node/app/dist), stage it at ./web where
# the server looks (see UI_DIR in crates/server/src/run.rs), then run the server
# serving both the API and the SPA on one origin — the way the Docker image does.
[group('dev')]
rest-ui *args: ui-build
    rm -rf web && cp -r node/app/dist web
    RUST_LOG=INFO cargo run -p olai-uc-server --features bin --bin uc-server -- serve {{ args }}

# build the bundled single-page app into node/app/dist
[group('build')]
ui-build:
    npm run build --workspace @open-lakehouse/uc-app

docs:
    npm run dev -w docs

# validate code examples type-check and docs build successfully
[group('test')]
validate-examples:
    cargo check -p unitycatalog-examples
    uvx ty check examples/python/
    npm run build -w @unitycatalog/client
    npx tsc --noEmit -p examples/typescript/tsconfig.json
    npm run build -w docs

# build python bindings
[group('build')]
build-py: build-py-client

# build python client bindings
[group('build')]
build-py-client:
    uv run maturin develop --uv --manifest-path python/client/Cargo.toml

# build a release manylinux wheel for one linux arch into dist/ (arch: x86_64 | aarch64).
# Cross-compiles with zig (`maturin build --zig`) so a single host builds both arches
# without qemu — the C deps (aws-lc-sys, via rustls) cross-compile cleanly under zig,
# whereas qemu emulation SIGSEGVs on them. The wheel is tagged manylinux_2_28 so it
# installs into the python:3.13-slim-bookworm marimo container; abi3 (pyo3 abi3-py39)
# means one wheel covers every Python >= 3.9.
#
# Host prereqs (one-time): `rustup target add x86_64-unknown-linux-gnu
# aarch64-unknown-linux-gnu`, `cargo install cargo-zigbuild`, and zig on PATH
# (`brew install zig`).
[group('build')]
build-py-wheel arch="x86_64":
    uv run maturin build --release --zig \
      --target {{ arch }}-unknown-linux-gnu \
      --manifest-path python/client/Cargo.toml \
      --out dist --compatibility manylinux_2_28

# build release manylinux wheels for both arches (amd64 + arm64) into dist/.
[group('build')]
build-py-wheels: (build-py-wheel "x86_64") (build-py-wheel "aarch64")

# build python server bindings
[group('build')]
build-py-server:
    uv run maturin develop --uv --manifest-path crates/cli/Cargo.toml

# build node bindings
[group('build')]
build-node:
    npm run build -w @unitycatalog/client

# build the server Docker image (hydrofoil): builds the bundled UI + the
# `uc-server` binary and assembles the distroless runtime with the SPA at ./web.
[group('build')]
build-docker:
    docker build -f Dockerfile -t hydrofoil:dev .

# Path to the committed UI fingerprint. It lives INSIDE the server crate so it is
# part of that crate's cargo-packaged file set — which is how release-plz decides
# a commit belongs to `olai-uc-server`. See `ui-fingerprint` below.
UI_LOCK := "crates/server/ui.lock"

# The UI is built into the Docker image (Dockerfile `ui` stage) but lives in
# node/, a sibling of the crate; release-plz only attributes a commit to
# `olai-uc-server` when a file in the crate's packaged set changes, and it cannot
# watch node/. This lock bridges that gap: it hashes every tracked file under
# node/, so any UI change moves the hash, the changed lock is a real change to a
# crate file, and a `feat`/`fix` commit carrying it makes release-plz bump
# `olai-uc-server` -> tag -> rebuild the Docker image (see release-plz.toml).
# Inputs come from `git ls-files` (deterministic; excludes gitignored build
# output / node_modules), sorted for a stable order across filesystems/locales.
#
# regenerate the UI fingerprint at {{ UI_LOCK }} after editing node/
[group('build')]
ui-fingerprint:
    #!/usr/bin/env bash
    set -euo pipefail
    hash=$(git ls-files node/ | LC_ALL=C sort | git hash-object --stdin-paths | sha256sum | cut -d' ' -f1)
    printf '# Fingerprint of the bundled Unity Catalog UI (node/), regenerated by\n# `just ui-fingerprint`. DO NOT EDIT BY HAND. This file lives in the\n# olai-uc-server crate so a UI change registers as a crate change and release-plz\n# rebuilds the Docker image — see the recipe and release-plz.toml.\n%s\n' "$hash" > {{ UI_LOCK }}
    echo "→ wrote {{ UI_LOCK }} ($hash)"

# CI runs this so a UI change can't merge without its regenerated lock — which is
# what makes the olai-uc-server version bump (and Docker rebuild) actually fire.
#
# fail if {{ UI_LOCK }} is stale relative to the current node/ tree
[group('test')]
ui-fingerprint-check:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(git ls-files node/ | LC_ALL=C sort | git hash-object --stdin-paths | sha256sum | cut -d' ' -f1)
    committed=$(grep -v '^#' {{ UI_LOCK }} | tr -d '[:space:]')
    if [ "$current" != "$committed" ]; then
        echo "::error::UI changed but {{ UI_LOCK }} is stale — run \`just ui-fingerprint\` and commit the result." >&2
        echo "  committed: $committed" >&2
        echo "  current:   $current" >&2
        exit 1
    fi
    echo "→ {{ UI_LOCK }} is up to date ($current)"

# build sqlx queries to support offline mode
[group('build')]
build-sqlx: _start_pg_sqlx
    # Wait for PostgreSQL to be ready
    sleep 1
    # Run migrations to create tables
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx migrate run --source ./crates/postgres/migrations
    # Prepare the postgres crate's queries from its OWN directory (not
    # `--workspace`), so the cache lands in `crates/postgres/.sqlx/` and travels
    # inside the published crate — `cargo publish`'s isolated verify build sees
    # it under SQLX_OFFLINE. (Mirrors `build-sqlx-sqlite`.)
    cd crates/postgres && DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo sqlx prepare -- --tests
    # Clean up
    @just _stop_pg_sqlx

# build sqlx queries for the embedded SQLite backend (no Docker required)
[group('build')]
build-sqlx-sqlite:
    #!/usr/bin/env bash
    set -euo pipefail
    DB="$(mktemp -t uc-sqlite-prepare-XXXX.db)"
    rm -f "$DB" "$DB"-wal "$DB"-shm
    export DATABASE_URL="sqlite://$DB"
    export SQLX_OFFLINE=false
    # Create the database file, apply the schema, then regenerate the offline
    # cache for the sqlite crate.
    cargo sqlx database create
    cargo sqlx migrate run --source ./crates/sqlite/migrations
    # Prepare only the sqlite crate's queries (run from its directory).
    cd crates/sqlite && cargo sqlx prepare -- --tests
    rm -f "$DB" "$DB"-wal "$DB"-shm

_start_pg_sqlx:
    docker run -d \
        --name unitycatalog-sqlx-pg \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_DB=postgres \
        -p 5432:5432 \
        postgres:16

_stop_pg_sqlx:
    docker stop unitycatalog-sqlx-pg && docker rm unitycatalog-sqlx-pg

[group('test')]
test-node:
    npm run test -w @unitycatalog/client

# run node integration tests (starts UC server automatically)
[group('test')]
test-node-integration:
    npm run build -w @unitycatalog/client
    npm run test:integration -w @unitycatalog/client

# run integration tests using mocked server responses
[group('test')]
integration:
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="false" \
    cargo run --bin unitycatalog-acceptance

# run journey tests live against the open-source Java Unity Catalog server.
# Boots the server via docker compose, waits for its healthcheck, then runs
# every OssJava-compatible journey. Tear down with:
# docker compose -f dev/uc-oss.compose.yaml down -v
[group('test')]
integration-oss-java:
    docker compose -f dev/uc-oss.compose.yaml up -d --wait
    UC_INTEGRATION_PROFILE="oss_java" \
    UC_INTEGRATION_URL="http://localhost:8080" \
    cargo test -p unitycatalog-acceptance -- journey_tests_live --nocapture

# Tear down afterwards with `docker compose -f dev/uc-oss.compose.yaml down -v`.
# Boot the Java OSS server and record fixtures into crates/acceptance/recordings/oss_java/
[group('test')]
record-oss-java:
    # docker compose -f dev/uc-oss.compose.yaml up -d --wait
    UC_INTEGRATION_PROFILE="oss_java" \
    UC_INTEGRATION_URL="http://localhost:9080" \
    UC_INTEGRATION_RECORD="true" \
    cargo test -p unitycatalog-acceptance -- journey_tests_live --nocapture
    # docker compose -f dev/uc-oss.compose.yaml down -v

# Boots the local Rust server in the background (shutting it down on exit) and
# records OssRust-compatible fixtures into crates/acceptance/recordings/oss_rust/
[group('test')]
record-oss-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --bin uc
    RUST_LOG=INFO cargo run --bin uc -- server --rest &
    server_pid=$!
    trap 'kill "$server_pid" 2>/dev/null || true' EXIT
    echo "⏳ Waiting for Rust server on http://localhost:8080 ..."
    for _ in $(seq 1 60); do
        if curl -sf -o /dev/null http://localhost:8080/api/2.1/unity-catalog/catalogs; then
            break
        fi
        sleep 1
    done
    UC_INTEGRATION_PROFILE="oss_rust" \
    UC_INTEGRATION_URL="http://localhost:8080" \
    UC_INTEGRATION_RECORD="true" \
    cargo test -p unitycatalog-acceptance -- journey_tests_live --nocapture

# run object-store integration tests against the docker `full` profile
# (UC server + SeaweedFS + Postgres + Azurite). Marks the test crate's
# `#[ignore]` tests as runnable.
[group('test')]
integration-object-store:
    docker compose -f dev/compose.yaml --profile full up -d
    UC_INTEGRATION_URL="http://localhost:8080/api/2.1/unity-catalog/" \
    cargo test -p olai-uc-object-store --test integration -- --ignored --test-threads=1

# run the credential-vending integration test against an Azurite sidecar.
# Boots the `azurite` compose profile (blob on localhost:10000), creates the
# `lakehouse` container the test expects (the vended SAS cannot create
# containers itself), then runs the `#[ignore]`d test under its feature gate.
[group('test')]
integration-azurite:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose -f dev/compose.yaml --profile azurite up -d --wait
    conn="DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://host.docker.internal:10000/devstoreaccount1;"
    # Pinned azure-cli: newer `az` defaults to a Storage API version no released
    # Azurite supports. 2.64.0's default is one Azurite accepts.
    docker run --rm mcr.microsoft.com/azure-cli:2.64.0 \
        az storage container create --name lakehouse --connection-string "$conn"
    UC_AZURITE_BLOB_ENDPOINT="http://127.0.0.1:10000" \
    UC_AZURITE_CONTAINER="lakehouse" \
    cargo test -p olai-uc-server --features integration-azurite \
        --test credential_vending_azurite -- --ignored --test-threads=1 --nocapture

[group('test')]
record-managed:
    UC_INTEGRATION_URL="$DATABRICKS_HOST" \
    UC_INTEGRATION_TOKEN="$DATABRICKS_TOKEN" \
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="true" \
    cargo run --bin unitycatalog-acceptance

# lint nodejs bindings
lint-node:
    npm run lint -w @unitycatalog/client

fix: fix-rust fix-node fix-py
    just fmt

# fix nodejs bindings
fix-node:
    npm run lint-fix -w @unitycatalog/client

# fix rust code
fix-rust:
    cargo clippy --fix --workspace --allow-dirty --allow-staged

fix-py:
    uvx ruff check --fix

fmt:
    cargo fmt
    buf format proto/ --write
    uvx ruff format .

asd:
    UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
    UC_TABLE=demo.managed_demo.events \
    AWS_REGION=eu-central-1 \
    cargo run -p olai-uc-datafusion --features delta --example managed_table_snapshot

fgh:
    AWS_REGION=eu-central-1 \
    cargo run -p olai-uc-datafusion --features delta --example managed_table_write
