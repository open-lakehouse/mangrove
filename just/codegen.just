# Code generation recipes.
#
# Most of the codebase is generated from the proto definitions in `proto/`.
# These recipes require the `trestle` CLI on PATH and (for the path patches) a
# `../trestle` source checkout — see the repo README / CONTRIBUTING for the full
# prerequisites. The fast edit/check loop (`cargo check`, `cargo test`) needs
# none of this tooling.
#
# All recipes anchor paths on `{{ justfile_directory() }}` (always the repo root,
# even from this module file) and carry `[no-cd]` where they use bare relative
# paths, so they behave exactly as they did at the repo root.

# The `olai-uc-server` crate embeds the OpenAPI specs via `include_str!` to serve
# Swagger UI. The canonical specs live at the repo root (`openapi/`), but a
# `cargo publish` tarball contains ONLY files under the crate, so an
# `../../../openapi/...` embed would break the published build. The sync recipes
# keep byte-identical copies inside the crate (`crates/server/openapi/`) that DO
# ship in the tarball.
SERVER_OPENAPI_DIR := "crates/server/openapi"

# run the full unitycatalog code-generation pipeline (proto + code + sharing + fix)
[group('codegen')]
generate: generate-proto generate-code generate-code-sharing
    just meta fix

# Generation of external types (google.api / gnostic extensions) now lives in the
# `../trestle` codegen repo (`olai-codegen`); see `generate-proto-gen-fixtures`.
#
# alias of `generate` (kept for muscle memory; identical pipeline)
[group('codegen')]
generate-full: generate-proto generate-code generate-code-sharing
    just meta fix

# run code generation for proto files (unitycatalog + sharing, incl. openapi)
[group('codegen')]
[no-cd]
generate-proto:
    buf generate proto/unitycatalog
    just codegen generate-openapi
    buf generate proto/sharing --template {{ justfile_directory() }}/buf.gen.sharing.yaml
    just codegen generate-openapi-sharing

# generate the Open Sharing OpenAPI spec from proto/sharing + merge NDJSON paths
[group('codegen')]
[no-cd]
generate-openapi-sharing:
    mkdir -p {{ justfile_directory() }}/openapi/sharing-gen
    buf generate proto/sharing --template {{ justfile_directory() }}/buf.gen.sharing-openapi.yaml
    uv run --with pyyaml python3 dev/scripts/merge_sharing_openapi.py \
      openapi/sharing-gen/openapi.yaml \
      openapi/sharing-query-paths.yaml \
      openapi/sharing.yaml
    rm -rf {{ justfile_directory() }}/openapi/sharing-gen
    bunx @redocly/cli bundle openapi/sharing.yaml > /dev/null

# update the generated openapi spec with validation extracted from generated jsonschema
[group('codegen')]
[no-cd]
generate-openapi:
    buf generate --template '{"version":"v2","plugins":[{"remote":"buf.build/bufbuild/protoschema-jsonschema:v0.6.0","opt": ["target=proto-strict-bundle"], "out":"openapi/jsonschema"}]}' proto
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    cargo run --manifest-path ../trestle/crates/trestle/Cargo.toml --bin trestle -- enrich-openapi \
      --config {{ justfile_directory() }}/trestle.yaml \
      --jsonschema-dir openapi/jsonschema \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm -f {{ justfile_directory() }}/descriptors.bin
    rm -rf openapi/jsonschema
    bunx @redocly/cli bundle --remove-unused-components openapi/openapi.yaml > tmp.yaml
    mv tmp.yaml openapi/openapi.yaml
    bun run openapi
    just codegen generate-node-openapi
    just codegen sync-server-openapi

# Regenerate the node unity-catalog-client's TypeScript API types from the
# canonical openapi/openapi.yaml (via openapi-typescript). Chained off
# generate-openapi so the client types can never drift from the spec; a CI drift
# guard (node-ci.yaml) fails the build if the committed output is stale.
#
# regenerate the node client's TypeScript API types from openapi/openapi.yaml
[group('codegen')]
[no-cd]
generate-node-openapi:
    cd {{ justfile_directory() }}/node/unity-catalog-client && bun run gen:api

# copy the canonical repo-root openapi specs into the server crate
[group('codegen')]
[no-cd]
sync-server-openapi:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p {{ justfile_directory() }}/{{ SERVER_OPENAPI_DIR }}
    for f in openapi.yaml delta.yaml; do
        cp {{ justfile_directory() }}/openapi/"$f" {{ justfile_directory() }}/{{ SERVER_OPENAPI_DIR }}/"$f"
    done
    echo "→ synced openapi.yaml, delta.yaml into {{ SERVER_OPENAPI_DIR }}"

# generate rest server and client code with the build crate (splices the PyO3 stub)
[group('codegen')]
[no-cd]
generate-code:
    buf build --output {{ justfile_directory() }}/descriptors.bin proto/unitycatalog
    trestle generate --config trestle.yaml \
      --descriptors {{ justfile_directory() }}/descriptors.bin
    rm {{ justfile_directory() }}/descriptors.bin
    just meta fmt
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

# The sharing surface lives in its own crates (`unitycatalog-sharing-client` for
# models + co-located extractors + client, `unitycatalog-sharing-api` for handler
# traits/routes), so it has its own trestle config (`trestle.sharing.yaml`)
# separate from the resource-oriented Unity Catalog pipeline in `generate-code`.
# The NDJSON table query RPCs are intentionally excluded from the proto service
# and implemented by hand (see `crates/sharing-client/src/query_extractors.rs`).
#
# generate sharing (Open Sharing) server/client/extractor code from proto/sharing
[group('codegen')]
[no-cd]
generate-code-sharing:
    buf build --output {{ justfile_directory() }}/sharing-descriptors.bin proto/sharing
    trestle generate --config trestle.sharing.yaml \
      --descriptors {{ justfile_directory() }}/sharing-descriptors.bin
    rm {{ justfile_directory() }}/sharing-descriptors.bin
    just meta fmt

# generate types for the node client (slow-changing external types)
[group('codegen')]
generate-node: generate-query-contract
    just node_client generate

# Generate TypeScript message types for the in-browser query contract
# (proto/query) into the @open-lakehouse/query package. Types only — no
# transport client; the runner is registered at runtime. See buf.gen.query.yaml.
#
# generate TypeScript message types for the in-browser query contract
[group('codegen')]
[no-cd]
generate-query-contract:
    buf generate proto/query --template {{ justfile_directory() }}/buf.gen.query.yaml

# Generate the volume-files contract (proto/portal/portal/files/v1) into
# crates/query-wasm/src/generated: buffa message types + the connect-rust
# FilesService trait/dispatcher (the in-wasm files engine implements it), AND the
# connect-es TypeScript client (message types + FilesService descriptor) into
# @open-lakehouse/files-connect. See buf.gen.files.yaml / buf.gen.files-es.yaml.
[group('codegen')]
[no-cd]
generate-files-proto: generate-files-es
    buf generate --template {{ justfile_directory() }}/buf.gen.files.yaml

# Generate the connect-es TypeScript client for the volume-files contract
# (message types + the FilesService service descriptor) into the
# @open-lakehouse/files-connect package. protoc-gen-es v2 emits the service
# descriptor directly; createClient(FilesService, transport) reads it. See
# buf.gen.files-es.yaml.
[group('codegen')]
[no-cd]
generate-files-es:
    buf generate --template {{ justfile_directory() }}/buf.gen.files-es.yaml

# regenerate proto-gen test fixture descriptors from proto/ source files
[group('codegen')]
[no-cd]
generate-proto-gen-fixtures:
    #!/usr/bin/env bash
    set -euo pipefail
    trestle_root="$({{ justfile_directory() }}/dev/scripts/trestle-root.sh)"
    buf dep update "$trestle_root/crates/trestle-codegen/proto"
    buf build --output "$trestle_root/crates/olai-codegen/proto/example.bin" \
      "$trestle_root/crates/olai-codegen/proto/"
