# Unity Catalog acceptance testing

An **API-coverage conformance battery** run against a live Unity Catalog server.
Each securable's lifecycle is a backend-agnostic check; the checks are gathered
into two batteries and run against a target (our Rust `uc-server`, the UC OSS
Java server, or managed Databricks).

This crate is a `publish = false` internal test harness and a library only —
there is no CLI binary. See [`JOURNEY_CATALOG.md`](./JOURNEY_CATALOG.md) for the
per-securable coverage inventory and the known-failing worklist.

## How it works

- **Checks** (`src/checks/*.rs`) — each is a free
  `pub async fn <name>(ctx: &JourneyContext) -> AcceptanceResult<()>` that drives
  one API surface end-to-end through `UnityCatalogClient` and asserts observable
  behavior. Checks self-name resources with a unique suffix and clean up on
  every path, so they are isolated from each other and from prior runs.
- **Batteries** (`src/conformance.rs`) — `baseline_checks()` (portable to any UC
  implementation, including UC OSS) and `extended_checks()` (everything our Rust
  server adds on top).
- **Run + quarantine** — `conformance::run` executes every check independently,
  captures each outcome (pass / skip / fail), and consults a per-`(target, check)`
  known-failing table. A quarantined check that fails is *expected* and does not
  break CI; a non-quarantined failure is a regression; a quarantined check that
  *passes* is flagged so the entry can be dropped. The printed inventory is the
  living record of which API surfaces work.
- **Entry points** (`tests/conformance.rs`) — three `#[tokio::test]`s, each gated
  on its target's URL env var and skipping when unset:
  - `conformance_oss_rust` — `UC_RUST_URL`; runs `extended_checks()`.
  - `conformance_oss_java` — `UC_OSS_JAVA_URL`; runs `baseline_checks()`.
  - `conformance_managed_databricks` — `UC_DATABRICKS_URL` + `_TOKEN`
    (+ `_STORAGE_ROOT`); runs `extended_checks()`. On-demand only.

## Running

```bash
# Against our own Rust server (boots it, runs the full battery under coverage):
just conformance-oss-rust

# Against the UC OSS Java server (docker compose):
just integration-oss-java
# tear down: docker compose -f dev/uc-oss.compose.yaml down -v

# Against managed Databricks (on-demand; never runs in CI):
UC_DATABRICKS_URL=https://…  UC_DATABRICKS_TOKEN=dapi…  \
  UC_DATABRICKS_STORAGE_ROOT=s3://bucket/uc-test/ \
  cargo test -p unitycatalog-acceptance -- conformance_managed_databricks --nocapture
```

Without any target env var set, `cargo test -p unitycatalog-acceptance` is a
no-op (all three tests skip) — which is why the plain workspace test run in CI
does not exercise a live server.

## Optional prerequisites for extended checks

Some extended checks self-skip unless configured:

| Variable | Enables |
|---|---|
| `UC_TEST_AWS_ROLE_ARN` / `UC_TEST_AZURE_ACCESS_CONNECTOR_ID` | `credential_lifecycle` |
| `UC_INTEGRATION_STORAGE_ROOT` (a `s3://` / `abfss://` / `gs://` root) | external-location / external-table / external-volume / path-credential / governance-chain checks |
| `UC_TEST_RECIPIENT_OWNER` | recipient owner on managed Databricks (defaults to `account users`) |
