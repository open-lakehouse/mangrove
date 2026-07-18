#!/usr/bin/env bash
# Seed the Azurite-backed managed-table preview environment.
#
# Prepares everything the in-browser wasm query engine needs to preview a real
# managed Delta table, then leaves the actual table write to the caller (the
# `seed_managed_tables` example — see `just ui-dev-wasm` / `just ui-dev`):
#
#   1. Start Azurite (docker compose `azurite` profile, blob on :10000).
#   2. Create the `lakehouse` blob container (a vended SAS cannot create it).
#   3. Add a CORS rule so the browser (served from the UC origin) may fetch
#      blobs from Azurite cross-origin — Azurite has no `--cors` flag, so this
#      is set at runtime via the Blob service properties.
#   4. Seed the UC storage credential + external location that credential
#      vending requires: `managed_storage_root` config places managed tables
#      but does NOT vend for them; vending needs an external location covering
#      the root, bound to an `azure_storage_key` credential.
#   5. Create the demo catalog + schema (managed root inherited from config).
#
# Idempotent: re-running is safe (already-exists responses are tolerated).
set -euo pipefail

# ── Configuration (override via env) ───────────────────────────────────────
UC_BASE="${UC_BASE:-http://localhost:8080/api/2.1/unity-catalog}"
CONTAINER="${UC_AZURITE_CONTAINER:-lakehouse}"
CATALOG="${UC_CATALOG:-demo}"
SCHEMA="${UC_SCHEMA:-default}"
COMPOSE_FILE="${COMPOSE_FILE:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/compose.yaml}"

# The well-known Azurite emulator account. The matching key is published by
# Microsoft and is deliberately not a secret, but to keep it out of this file
# we source it from the repo's existing test constant (or an env override).
# It's the same value the `integration-azurite` recipe / CI already use.
ACCOUNT="devstoreaccount1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACCOUNT_KEY="${AZURITE_ACCOUNT_KEY:-$(
  grep -oE '[A-Za-z0-9+/]{80,}==' \
    "${SCRIPT_DIR}/../../crates/server/tests/credential_vending_azurite.rs" | head -1
)}"
if [ -z "${ACCOUNT_KEY}" ]; then
  echo "[seed] could not resolve the Azurite account key; set AZURITE_ACCOUNT_KEY" >&2
  exit 1
fi
# az-cli pinned: newer versions default to a Storage API version Azurite rejects.
AZ_CLI_IMAGE="mcr.microsoft.com/azure-cli:2.64.0"
# The az-cli runs in a container, so it reaches the host's Azurite via
# host.docker.internal (matches the `integration-azurite` recipe).
AZ_CONN="DefaultEndpointsProtocol=http;AccountName=${ACCOUNT};AccountKey=${ACCOUNT_KEY};BlobEndpoint=http://host.docker.internal:10000/${ACCOUNT};"

log() { printf '\033[1;34m[seed]\033[0m %s\n' "$*"; }

# ── 1. Azurite ─────────────────────────────────────────────────────────────
# Start ONLY the azurite service by name — `--profile azurite up` would also
# start the profile-less postgres_uc_dev (it belongs to the default profile),
# which the preview flow does not need and whose :5432 often clashes.
log "starting Azurite (docker compose azurite_uc_dev)…"
docker compose -f "$COMPOSE_FILE" --profile azurite up -d --wait azurite_uc_dev

# ── 2. Container ───────────────────────────────────────────────────────────
log "ensuring blob container '${CONTAINER}' exists…"
docker run --rm "$AZ_CLI_IMAGE" \
  az storage container create --name "$CONTAINER" --connection-string "$AZ_CONN" \
  >/dev/null

# ── 3. CORS (so the browser wasm engine can fetch blobs cross-origin) ───────
log "adding permissive CORS rule to Azurite blob service…"
docker run --rm "$AZ_CLI_IMAGE" \
  az storage cors add --services b \
    --methods GET HEAD OPTIONS \
    --origins '*' --allowed-headers '*' --exposed-headers '*' --max-age 3600 \
    --connection-string "$AZ_CONN" \
  >/dev/null

# ── Wait for the UC server to answer ───────────────────────────────────────
log "waiting for UC server at ${UC_BASE}…"
for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "${UC_BASE}/catalogs"; then break; fi
  sleep 1
done

# POST helper: succeed on 2xx, tolerate an already-exists 409/400 (idempotent).
post() {
  local path="$1" body="$2" desc="$3" code
  code=$(curl -s -o /tmp/seed-resp.$$ -w '%{http_code}' \
    -X POST "${UC_BASE}${path}" -H 'Content-Type: application/json' -d "$body")
  case "$code" in
    2*) log "  ${desc}: created" ;;
    409|400) log "  ${desc}: already exists (${code}), continuing" ;;
    *) log "  ${desc}: FAILED (${code})"; cat /tmp/seed-resp.$$; rm -f /tmp/seed-resp.$$; exit 1 ;;
  esac
  rm -f /tmp/seed-resp.$$
}

# ── 4. Storage credential + external location ──────────────────────────────
log "seeding storage credential + external location…"
post /credentials "$(cat <<JSON
{"name":"azurite_key","purpose":"STORAGE","skipValidation":true,
 "azureStorageKey":{"accountName":"${ACCOUNT}","accountKey":"${ACCOUNT_KEY}"}}
JSON
)" "credential azurite_key"

post /external-locations "$(cat <<JSON
{"name":"azurite_loc","url":"azurite://${CONTAINER}","credentialName":"azurite_key","skipValidation":true}
JSON
)" "external location azurite_loc"

# ── 5. Catalogs + schemas (managed root inherited from server config) ──────
# Two catalogs with a handful of schemas, all commented, to give the UI a
# non-trivial namespace tree. The `${CATALOG}`/`${SCHEMA}` env pair still
# seeds its own catalog+schema (defaults demo/default) for standalone use; the
# richer `demo`/`ml` layout below is what the managed-table seed writes into.
log "creating catalogs + schemas…"
post /catalogs \
  "{\"name\":\"${CATALOG}\",\"comment\":\"Demo analytics catalog (customers, orders, sales).\"}" \
  "catalog ${CATALOG}"
post /schemas \
  "{\"name\":\"${SCHEMA}\",\"catalogName\":\"${CATALOG}\",\"comment\":\"Default schema — core demo tables.\"}" \
  "schema ${CATALOG}.${SCHEMA}"
post /schemas \
  "{\"name\":\"sales\",\"catalogName\":\"${CATALOG}\",\"comment\":\"Sales aggregates and region dimensions.\"}" \
  "schema ${CATALOG}.sales"

post /catalogs \
  "{\"name\":\"ml\",\"comment\":\"Machine-learning assets (features + registered models).\"}" \
  "catalog ml"
post /schemas \
  "{\"name\":\"features\",\"catalogName\":\"ml\",\"comment\":\"Feature tables and model registry.\"}" \
  "schema ml.features"

# ── 6. Volumes (MANAGED — location derived under the managed root) ─────────
log "seeding volumes…"
post /volumes \
  "{\"catalogName\":\"${CATALOG}\",\"schemaName\":\"${SCHEMA}\",\"name\":\"raw_files\",\"volumeType\":\"MANAGED\",\"comment\":\"Landing zone for raw uploads.\"}" \
  "volume ${CATALOG}.${SCHEMA}.raw_files"
post /volumes \
  "{\"catalogName\":\"ml\",\"schemaName\":\"features\",\"name\":\"artifacts\",\"volumeType\":\"MANAGED\",\"comment\":\"Model artifacts and training outputs.\"}" \
  "volume ml.features.artifacts"

# ── 7. Functions (SQL, metadata-only; body wrapped in `functionInfo`) ──────
log "seeding functions…"
post /functions "$(cat <<JSON
{"functionInfo":{
  "name":"add_tax","catalogName":"${CATALOG}","schemaName":"${SCHEMA}",
  "dataType":"DOUBLE","fullDataType":"DOUBLE","parameterStyle":"S",
  "isDeterministic":true,"sqlDataAccess":"NO_SQL","isNullCall":false,
  "securityType":"DEFINER","routineBody":"SQL",
  "routineDefinition":"amount * 1.2",
  "comment":"Gross-up an amount by a flat 20% tax rate."}}
JSON
)" "function ${CATALOG}.${SCHEMA}.add_tax"
post /functions "$(cat <<JSON
{"functionInfo":{
  "name":"fiscal_quarter","catalogName":"${CATALOG}","schemaName":"sales",
  "dataType":"INT","fullDataType":"INT","parameterStyle":"S",
  "isDeterministic":true,"sqlDataAccess":"NO_SQL","isNullCall":false,
  "securityType":"DEFINER","routineBody":"SQL",
  "routineDefinition":"floor((month(d) - 1) / 3) + 1",
  "comment":"Fiscal quarter (1-4) for a given date."}}
JSON
)" "function ${CATALOG}.sales.fiscal_quarter"

# ── 8. Registered models + versions (metadata-only) ────────────────────────
log "seeding registered models + versions…"
post /models \
  "{\"name\":\"churn_predictor\",\"catalogName\":\"ml\",\"schemaName\":\"features\",\"comment\":\"Predicts customer churn probability.\"}" \
  "model ml.features.churn_predictor"
post /models \
  "{\"name\":\"recommender\",\"catalogName\":\"ml\",\"schemaName\":\"features\",\"comment\":\"Product recommendation model.\"}" \
  "model ml.features.recommender"

# `source` is a stand-in artifact URI — no artifact upload is needed for the
# metadata-only create. Versions start in PENDING_REGISTRATION, which the UI
# lists fine; no finalize call required.
post /models/versions \
  "{\"modelName\":\"churn_predictor\",\"catalogName\":\"ml\",\"schemaName\":\"features\",\"source\":\"azurite://lakehouse/models/churn_predictor/1\",\"comment\":\"Baseline logistic regression.\"}" \
  "model version churn_predictor/1"
post /models/versions \
  "{\"modelName\":\"churn_predictor\",\"catalogName\":\"ml\",\"schemaName\":\"features\",\"source\":\"azurite://lakehouse/models/churn_predictor/2\",\"comment\":\"Gradient-boosted retrain.\"}" \
  "model version churn_predictor/2"
post /models/versions \
  "{\"modelName\":\"recommender\",\"catalogName\":\"ml\",\"schemaName\":\"features\",\"source\":\"azurite://lakehouse/models/recommender/1\",\"comment\":\"Initial collaborative-filtering model.\"}" \
  "model version recommender/1"

log "done. managed tables written under demo/ml will be preview-able."
