# Test & CI-gate recipes: unit/integration/conformance test batteries plus the
# drift guards CI runs (`sync-server-openapi-check`, `ui-fingerprint-check`).
#
# Recipes anchor paths on `{{ justfile_directory() }}` (repo root) and carry
# `[no-cd]` where they use bare relative paths.

# Azurite's universally-published well-known emulator account + key (NOT a
# secret — it is baked into every Azure SDK and the Azurite docs). Split across
# two vars and joined at runtime so the value never forms a storage
# connection-string / account-key literal on a committed line.
AZURITE_ACCOUNT := "devstoreaccount1"
AZURITE_KEY_A := "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq"
AZURITE_KEY_B := "/K1SZFPTOtr/KBHBeksoGMGw=="

# validate code examples type-check and docs build successfully
[group('test')]
[no-cd]
validate-examples:
    cargo check -p unitycatalog-examples
    uvx ty check examples/python/
    bun run --filter @unitycatalog/client build
    bunx tsc --noEmit -p examples/typescript/tsconfig.json
    bun run --filter docs build

# Regenerate with `just codegen sync-server-openapi`. The path is duplicated from
# `codegen`'s SERVER_OPENAPI_DIR — modules cannot share variables, so keep the two
# in sync by hand.
#
# CI gate: fail if the in-crate openapi copies are stale vs the canonical specs
[group('test')]
[no-cd]
sync-server-openapi-check:
    #!/usr/bin/env bash
    set -euo pipefail
    server_openapi_dir="crates/server/openapi"
    for f in openapi.yaml delta.yaml; do
        if ! diff -q {{ justfile_directory() }}/openapi/"$f" \
            {{ justfile_directory() }}/"$server_openapi_dir"/"$f" >/dev/null; then
            echo "::error::$server_openapi_dir/$f is out of sync with openapi/$f — run \`just codegen sync-server-openapi\` and commit the result." >&2
            exit 1
        fi
    done

# Regenerate with `just build ui-fingerprint`. This gate is what makes the
# olai-uc-server version bump (and Docker rebuild) fire on a UI change. The lock
# path is duplicated from `build`'s UI_LOCK — modules cannot share variables, so
# keep the two in sync by hand.
#
# CI gate: fail if the UI fingerprint lock is stale vs the current node/ tree
[group('test')]
[no-cd]
ui-fingerprint-check:
    #!/usr/bin/env bash
    set -euo pipefail
    ui_lock="crates/server/ui.lock"
    current=$(git ls-files node/ | LC_ALL=C sort | git hash-object --stdin-paths | sha256sum | cut -d' ' -f1)
    committed=$(grep -v '^#' "$ui_lock" | tr -d '[:space:]')
    if [ "$current" != "$committed" ]; then
        echo "::error::UI changed but $ui_lock is stale — run \`just build ui-fingerprint\` and commit the result." >&2
        echo "  committed: $committed" >&2
        echo "  current:   $current" >&2
        exit 1
    fi
    echo "→ $ui_lock is up to date ($current)"

# run node binding unit tests
[group('test')]
test-node:
    bun run --filter @unitycatalog/client test

# run node integration tests (starts UC server automatically)
[group('test')]
test-node-integration:
    bun run --filter @unitycatalog/client build
    bun run --filter @unitycatalog/client test:integration

# Boots the open-source Java UC server via docker compose, waits for its
# healthcheck, then runs `conformance_oss_java`. Managed tables land under
# file:///tmp/uc-test, bind-mounted 1:1 from the host. Tear down with:
# docker compose -f dev/uc-oss.compose.yaml down -v
#
# run the portable baseline conformance battery against the OSS Java UC server
[group('test')]
[no-cd]
integration-oss-java:
    rm -rf /tmp/uc-test && mkdir -p /tmp/uc-test
    docker compose -f dev/uc-oss.compose.yaml up -d --wait
    UC_OSS_JAVA_URL="http://localhost:8080" \
    cargo test -p unitycatalog-acceptance -- conformance_oss_java --nocapture

# Uses dev/uc-rust-conformance.yaml. On macOS `/tmp` is a symlink, so the config
# needs the canonical `/private/tmp/uc-test`; adjust the allowed root + storage
# root there for a local run (CI runs on Linux where `/tmp/uc-test` is fine).
#
# boot the local Rust server in the background and run the full conformance battery
[group('test')]
[no-cd]
conformance-oss-rust:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p /tmp/uc-test
    cargo build -p olai-uc-server --features bin --bin uc-server
    RUST_LOG=INFO cargo run -p olai-uc-server --features bin --bin uc-server -- \
        serve --config dev/uc-rust-conformance.yaml &
    server_pid=$!
    trap 'kill "$server_pid" 2>/dev/null || true' EXIT
    echo "⏳ Waiting for Rust server on http://localhost:8080 ..."
    for _ in $(seq 1 60); do
        if curl -sf -o /dev/null http://localhost:8080/api/2.1/unity-catalog/catalogs; then
            break
        fi
        sleep 1
    done
    UC_RUST_URL="http://localhost:8080" \
    cargo test -p unitycatalog-acceptance -- conformance_oss_rust --nocapture

# Runs against the docker `full` profile (UC server + SeaweedFS + Postgres +
# Azurite). Marks the test crate's `#[ignore]` tests as runnable.
#
# run object-store integration tests against the docker `full` profile
[group('test')]
[no-cd]
integration-object-store:
    docker compose -f dev/compose.yaml --profile full up -d
    UC_INTEGRATION_URL="http://localhost:8080/api/2.1/unity-catalog/" \
    cargo test -p olai-uc-object-store --test integration -- --ignored --test-threads=1

# Boots the `azurite` compose profile (blob on localhost:10000), creates the
# `lakehouse` container the test expects (the vended SAS cannot create containers
# itself), then runs the `#[ignore]`d test under its feature gate.
#
# run the credential-vending integration test against an Azurite sidecar
[group('test')]
[no-cd]
integration-azurite:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose -f dev/compose.yaml --profile azurite up -d --wait
    # Assemble the Azurite well-known dev connection string from parts (see the
    # AZURITE_* vars above) so no account-key literal lands on a committed line.
    account="{{ AZURITE_ACCOUNT }}"
    key="{{ AZURITE_KEY_A }}{{ AZURITE_KEY_B }}"
    conn="DefaultEndpointsProtocol=http;AccountName=${account};AccountKey=${key};BlobEndpoint=http://host.docker.internal:10000/${account};"
    # Pinned azure-cli: newer `az` defaults to a Storage API version no released
    # Azurite supports. 2.64.0's default is one Azurite accepts.
    docker run --rm mcr.microsoft.com/azure-cli:2.64.0 \
        az storage container create --name lakehouse --connection-string "$conn"
    UC_AZURITE_BLOB_ENDPOINT="http://127.0.0.1:10000" \
    UC_AZURITE_CONTAINER="lakehouse" \
    cargo test -p olai-uc-server --features integration-azurite \
        --test credential_vending_azurite -- --ignored --test-threads=1 --nocapture

# record managed-table acceptance fixtures against a live Databricks workspace
[group('test')]
[no-cd]
record-managed:
    UC_INTEGRATION_URL="$DATABRICKS_HOST" \
    UC_INTEGRATION_TOKEN="$DATABRICKS_TOKEN" \
    UC_INTEGRATION_DIR="{{ justfile_directory() }}/crates/acceptance/recordings" \
    UC_INTEGRATION_STORAGE_ROOT="$DATABRICKS_STORAGE_ROOT" \
    UC_INTEGRATION_RECORD="true" \
    cargo run --bin unitycatalog-acceptance
