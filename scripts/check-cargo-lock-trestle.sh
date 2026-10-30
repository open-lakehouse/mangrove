#!/usr/bin/env bash
# Verify Cargo.lock lists the trestle crates (olai-*) from crates.io, not local path patches.
#
# `just configure-trestle-deps` writes a gitignored [patch.crates-io] that resolves
# these crates from ../trestle. If Cargo.lock is regenerated under that patch, the
# registry source/checksum lines disappear and CI (--locked) fails.
#
# Usage:
#   scripts/check-cargo-lock-trestle.sh              # check ./Cargo.lock
#   scripts/check-cargo-lock-trestle.sh path/to/Cargo.lock
set -euo pipefail

LOCK="${1:-Cargo.lock}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCK_PATH="$LOCK"
if [[ "$LOCK" != /* ]]; then
  LOCK_PATH="$REPO_ROOT/$LOCK"
fi

if [[ ! -f "$LOCK_PATH" ]]; then
  echo "error: $LOCK_PATH not found" >&2
  exit 1
fi

extract_package_block() {
  local pkg=$1
  awk -v pkg="$pkg" '
    # Any `[[...]]` header ends a block; only `[[package]]` blocks are matched
    # (so a `[[patch.unused]]` entry never counts as a registry package).
    /^\[\[/ {
      if (block ~ ("name = \"" pkg "\"")) {
        print block
        exit
      }
      block = ($0 == "[[package]]") ? $0 "\n" : ""
      next
    }
    { block = block $0 "\n" }
    END {
      if (block ~ ("name = \"" pkg "\"")) print block
    }
  ' "$LOCK_PATH"
}

fail() {
  echo "error: $*" >&2
  echo "  Local trestle path patches (just configure-trestle-deps) must not alter committed Cargo.lock." >&2
  echo "  Restore from main: git checkout main -- Cargo.lock" >&2
  echo "  Or regenerate without patches:" >&2
  echo "    mv .cargo/config.toml /tmp/cargo-config.toml.bak" >&2
  echo "    cargo generate-lockfile" >&2
  echo "    mv /tmp/cargo-config.toml.bak .cargo/config.toml" >&2
  exit 1
}

# Every crate `just configure-trestle-deps` patches. A lockfile need not contain all
# of them (the `wasm/` workspace has no olai-store), so absent ones are skipped.
TRESTLE_CRATES=(
  olai-credential-vending
  olai-http
  olai-http-wasm
  olai-storage-credentials
  olai-store
  olai-vended-store
)

found=0
for pkg in "${TRESTLE_CRATES[@]}"; do
  block=$(extract_package_block "$pkg")
  if [[ -z "$block" ]]; then
    continue
  fi
  found=$((found + 1))
  if ! grep -q '^source = "registry+' <<<"$block"; then
    fail "$pkg is missing a crates.io source line (lockfile was likely generated with a local trestle patch)"
  fi
  if ! grep -q '^checksum = "' <<<"$block"; then
    fail "$pkg is missing a crates.io checksum line"
  fi
done

if [[ $found -eq 0 ]]; then
  fail "$LOCK_PATH lists none of the trestle crates (${TRESTLE_CRATES[*]})"
fi

echo "$LOCK: all $found trestle crates have crates.io registry entries."
