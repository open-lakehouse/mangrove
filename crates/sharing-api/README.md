# olai-uc-sharing-api

Portable Unity Catalog **Open Sharing** / **Delta Sharing** REST API.

This crate owns the sharing *semantics* — share / schema / asset discovery,
the table query path (version / metadata / query) and the storage-backed asset
surface (volumes, agent skills) — and expresses them over three narrow ports:

- `ShareCatalog`: which shares and objects a recipient sees, and where a shared
  object lives (backed by any source catalog);
- `StorageAccess`: the server's own reads of a shared asset, plus presigned file
  URLs and temporary credentials for recipients;
- `TableFormat`: serving one table format. Delta is built in (feature `delta`)
  and reads tables through `delta_kernel` directly, so no query engine is
  involved.

`SharingService` composes one implementation of each port and implements every
handler. `sharing_router` serves it under `/api/v1/delta-sharing` (Delta
Sharing) and `/api/1.0/opensharing` (Open Sharing, adding volumes, agent skills
and temporary credentials), and `auth::require_recipient` puts a bearer-token
`RecipientAuthenticator` in front. The wire types come from
[`olai-uc-sharing-client`](https://crates.io/crates/olai-uc-sharing-client).

## Status

- **Discovery** (shares, schemas, tables, volumes, skills) is implemented and
  paginated.
- **Query** (`version`, `metadata`, `query`) serves Delta tables in the parquet
  response format, with version / timestamp time travel, predicate-hint file
  skipping and limit hints. Tables that need reader features (column mapping,
  deletion vectors, …) require `responseformat=delta` and are rejected for now.
- **Credentials**: `temporary-table-credentials` (Open Sharing) and the volume /
  skill credential endpoints vend read-only credentials through `StorageAccess`.
- **Not implemented** (`501`): change data feed (`/changes`), asynchronous
  queries (`POST /queries/{id}`) and `responseformat=delta` serving.

## Architecture

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the pluggable server design
(ports, source-catalog share mapping, delegated vs. local storage access,
secrets at rest) and the gap assessment against the upstream OpenSharing
reference implementation.
