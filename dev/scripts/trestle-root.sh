#!/usr/bin/env bash
# Print the absolute path to a trestle *source* checkout (needed for olai-http /
# olai-store path patches and bundled migrations). The `trestle` CLI on PATH
# (Homebrew / release binary) is not enough on its own — it ships only the binary.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ -n "${TRESTLE_ROOT:-}" ]]; then
    root="$TRESTLE_ROOT"
elif [[ -d "$repo_root/../trestle/crates/olai-http" ]]; then
    root="$(cd "$repo_root/../trestle" && pwd)"
else
    echo "error: could not locate a trestle source checkout." >&2
    echo "  Clone https://github.com/open-lakehouse/trestle adjacent to mangrove (../trestle)," >&2
    echo "  or set TRESTLE_ROOT to the checkout path." >&2
    if command -v trestle >/dev/null 2>&1; then
        echo "  Note: $(command -v trestle) is the codegen CLI only;" >&2
        echo "  olai-http/olai-store must be patched from a source checkout." >&2
    fi
    exit 1
fi

if [[ ! -f "$root/crates/olai-http/Cargo.toml" || ! -f "$root/crates/olai-store/Cargo.toml" ]]; then
    echo "error: trestle checkout at $root is missing olai-http/olai-store crates" >&2
    exit 1
fi

printf '%s\n' "$root"
