#!/usr/bin/env bash
# Apply the *consolidated* SQLite schema to $DATABASE_URL, matching the runtime
# `unified_migrator()` in `crates/sqlite/src/store.rs`: olai-store's object-graph
# migrations (0001+) first, then the mangrove-local migrations (`delta_commits`).
#
# `cargo sqlx prepare` for the sqlite crate recompiles `olai_store` too, so its
# `sqlx::query!` macros probe this DB live — the `objects`/`associations` tables
# olai-store owns must exist or the build fails with "no such table". olai-store
# ships those migrations *inside the crate*, so we locate them via `cargo
# metadata` — that resolves whether olai-store comes from the crates.io registry
# (CI) or a local `[patch.crates-io]` -> ../trestle checkout (local dev).
#
# olai-store's SQL is applied raw (not via `sqlx migrate run`) so its files don't
# register in the `_sqlx_migrations` ledger and collide with the mangrove-local
# migrator that shares this one database — the same split the runtime's
# `sql_migrator_with` performs. olai-store's statements are all `IF NOT EXISTS`,
# so the raw apply is idempotent.
#
# Requires: $DATABASE_URL set to a `sqlite://<path>` URL; `cargo`, `sqlite3`, and
# `sqlx` (sqlx-cli) on PATH.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

: "${DATABASE_URL:?DATABASE_URL must be set to a sqlite:// URL}"
db_path="${DATABASE_URL#sqlite://}"

# Locate olai-store's manifest, then its bundled sqlite migrations, wherever
# cargo resolved the crate (registry cache or local patch path).
olai_manifest="$(cargo metadata --format-version 1 \
    | python3 -c "import sys,json;m=json.load(sys.stdin);print(next(p['manifest_path'] for p in m['packages'] if p['name']=='olai-store'))")"
olai_migrations="$(dirname "$olai_manifest")/migrations/sqlite"

if [ ! -d "$olai_migrations" ]; then
    echo "error: olai-store sqlite migrations not found at $olai_migrations" >&2
    exit 1
fi

echo "Applying olai-store sqlite migrations from $olai_migrations"
cat "$olai_migrations"/*.sql | sqlite3 "$db_path"

echo "Applying mangrove-local sqlite migrations"
cargo sqlx migrate run --source ./crates/sqlite/migrations
