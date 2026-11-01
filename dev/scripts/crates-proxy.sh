#!/usr/bin/env bash
# Run a trestle crates-proxy script (patch, lock-registry, check-lock, upstream-root).
# See https://github.com/open-lakehouse/trestle/blob/main/docs/crates-proxy.md
# --if-available: exit 0 with a note when no trestle checkout is found (for hooks;
# CI's --locked build is the backstop).
set -euo pipefail
optional=0
if [[ "${1:-}" == "--if-available" ]]; then optional=1; shift; fi
cmd="${1:?usage: crates-proxy.sh [--if-available] <patch|lock-registry|check-lock|upstream-root> [args]}"
shift
if [[ -n "${TRESTLE_ROOT:-}" ]]; then
    root="$TRESTLE_ROOT"
else
    main="$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"
    root="$(dirname "$main")/trestle"
fi
script="$root/scripts/crates-proxy/$cmd.sh"
if [[ ! -x "$script" ]]; then
    if [[ $optional -eq 1 ]]; then
        echo "note: no trestle checkout at $root; skipping crates-proxy $cmd" >&2
        exit 0
    fi
    echo "error: $script not found; clone trestle next to this repo or set TRESTLE_ROOT" >&2
    exit 1
fi
exec "$script" "$@"
