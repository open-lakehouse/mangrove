# olai-uc-server

**Unity Catalog REST server with pluggable storage backends — a
deployable catalog service and the library it is built from.**

> [!NOTE]
> The Rust crate identifier is `unitycatalog_server` (imports and paths use that
> name); the crate is published to crates.io as `olai-uc-server` while the naming
> settles. This is an experimental component of an unofficial
> [Unity Catalog](https://unitycatalog.io) implementation.

This crate is two things at once:

- A **library** (`unitycatalog_server`) providing the Unity Catalog REST
  handlers, routing, storage services, and the reusable handler patterns
  (in-memory backend, proxy/federation connectors) that a custom deployment can
  build on.
- A **deployable binary**, `uc-server` (gated behind the `bin` feature), with
  `serve` / `migrate` / `healthcheck` subcommands. This is the same binary that
  ships as the [`mangrove`](https://github.com/open-lakehouse/mangrove) Docker
  image (`ghcr.io/open-lakehouse/mangrove`) with the web UI bundled in.

The `uc-server` binary is distinct from the `uc` client CLI (crate
`olai-uc-cli`): `uc-server` *runs* a catalog, `uc` *talks to* one.

## Install the server binary

The binary is gated behind the `bin` feature (off by default, so library
consumers don't pull the serve/store/CLI stack), so enable it when installing:

```sh
cargo install olai-uc-server --features bin
```

This installs `uc-server`:

```sh
uc-server serve            # run the catalog server (REST + bundled UI)
uc-server migrate          # run database migrations
uc-server healthcheck      # probe /health (used by the Docker HEALTHCHECK)
```

`serve` loads a YAML config file (also `UC_SERVER_CONFIG`), overlaid by CLI flags
(`--host` / `--port` / `--no-ui`, CLI winning).

Prefer a container? Pull the image instead:

```sh
docker run -p 8080:8080 ghcr.io/open-lakehouse/mangrove:latest
```

## Use as a library

```toml
[dependencies]
# The `package` rename keeps the `unitycatalog_server` import path.
unitycatalog-server = { package = "olai-uc-server", version = "0.0.2" }
```

A bare library build brings in only the handler/routing surface (default
features `axum` + `memory`); the `bin` feature and its storage-backend /
CLI / UI-serving dependencies stay out unless you opt in.

## Feature flags

| Feature   | Default | What it enables                                                                          |
| --------- | :-----: | ---------------------------------------------------------------------------------------- |
| `axum`    |   yes   | The axum router, extractors, and served HTTP surface.                                    |
| `memory`  |   yes   | The in-memory handler backend (random/time-ordered IDs).                                 |
| `proxy`   |         | Reusable proxy leaves + a Unity Catalog client connector.                                |
| `federation` |      | Federation over the proxy connector (implies `proxy`).                                   |
| `bin`     |         | The deployable `uc-server` binary: SQLite/Postgres backends, the hybrid proxy, swagger UI, the CLI parser, and the healthcheck client. |

## Status

Experimental and pre-1.0; the API surface may change. Part of the
[mangrove](https://github.com/open-lakehouse/mangrove) workspace.

## License

Licensed under the Apache License, Version 2.0.
