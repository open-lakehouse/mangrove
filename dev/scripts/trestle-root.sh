#!/usr/bin/env bash
# Print the absolute path to a trestle *source* checkout (needed for olai-http /
# olai-store path patches and bundled migrations). The `trestle` CLI on PATH
# (Homebrew / release binary) is not enough on its own — it ships only the binary.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Resolve ../trestle next to the *main* checkout, so this also works from a linked
# worktree (e.g. ~/code/mangrove-worktrees/<branch>, which has no ../trestle).
main_root="$(dirname "$(git -C "$repo_root" rev-parse --path-format=absolute --git-common-dir)")"

if [[ -n "${TRESTLE_ROOT:-}" ]]; then
    root="$TRESTLE_ROOT"
elif [[ -d "$main_root/../trestle/crates/olai-http" ]]; then
    root="$(cd "$main_root/../trestle" && pwd)"
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
