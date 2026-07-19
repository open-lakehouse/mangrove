set dotenv-load

# ─── Task modules ─────────────────────────────────────────────────────────────
# Recipes are organized into modules by concern. Invoke them namespaced, e.g.
# `just codegen generate`, `just build ui-build`, `just test test-node`,
# `just dev rest`, `just meta fmt`. Run `just <module> --list` to see a module's
# recipes. A handful of names CI and the docs depend on are re-exported flat at
# the bottom of this file as aliases.

# proto/openapi/client code generation
mod codegen 'just/codegen.just'
# UI, bindings, wasm engine, Docker image, sqlx caches
mod build 'just/build.just'
# unit / integration / conformance tests + CI drift guards
mod test 'just/test.just'
# formatting, autofix, linting, docs, `run`
mod meta 'just/meta.just'
# docker-compose dev environment + UC server / UI dev flows
mod dev 'dev/justfile'
# olai-uc-common proto ext generation
mod common 'crates/common/justfile'
# postgres sharing migrations helpers
mod postgres 'crates/postgres/justfile'
# node client external-type generation
mod node_client 'node/client/justfile'

# show the available modules and recipes
_default:
    @just --list

# ─── Back-compat aliases ──────────────────────────────────────────────────────
# Flat names that CI (.github/workflows/*.yaml) and .github/CONTRIBUTING.md refer
# to, kept working after the module split. Prefer the namespaced form for new use.
alias generate := codegen::generate
alias generate-proto := codegen::generate-proto
alias generate-code := codegen::generate-code
alias generate-openapi := codegen::generate-openapi
alias ui-build := build::ui-build
alias ui-fingerprint := build::ui-fingerprint
alias ui-fingerprint-check := test::ui-fingerprint-check
alias sync-server-openapi-check := test::sync-server-openapi-check
alias rest := dev::rest
alias rest-ui := dev::rest-ui
alias configure-trestle-deps := dev::configure-trestle-deps
alias fmt := meta::fmt
alias fix := meta::fix
