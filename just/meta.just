# Cross-cutting helper recipes: formatting, autofix, linting, docs, and the
# generic `run` passthrough. Called by codegen (`fmt`/`fix`) and used directly.

# run the uc CLI binary (passes through extra args)
[group('meta')]
run *args:
    cargo run --bin uc {{ args }}

# format all code (Rust, protobuf, Python)
[group('meta')]
[no-cd]
fmt:
    cargo fmt
    buf format proto/ --write
    uvx ruff format .

# auto-fix Rust, Node, and Python lint issues, then format
[group('meta')]
fix: fix-rust fix-node fix-py
    just meta fmt

# auto-fix Rust lint issues
[group('meta')]
fix-rust:
    cargo clippy --fix --workspace --allow-dirty --allow-staged

# auto-fix Node lint issues (biome across the whole node/ workspace)
[group('meta')]
[no-cd]
fix-node:
    bunx biome check --write node

# auto-fix Python lint issues
[group('meta')]
[no-cd]
fix-py:
    uvx ruff check --fix

# lint + typecheck the node/ workspace (mirrors the node-ci `ui` job)
[group('meta')]
[no-cd]
lint-node:
    bun run lint:node
    bun run typecheck

