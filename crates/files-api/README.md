# olai-uc-files-api

A ConnectRPC `portal.files.v1.FilesService` server for Unity Catalog volume
files — upload, download, delete, metadata, and directory listing over
Databricks Volumes paths (`/Volumes/<catalog>/<schema>/<volume>/…`).

Like [`olai-uc-storage-proxy`](../storage-proxy), it runs in two modes from one
crate:

1. **Embedded** in the Unity Catalog server (`olai-uc-server`). The `server`
   feature exposes `AppState` and the generated service impl; the host builds a
   ConnectRPC router, turns it into an axum fallback service, and nests it under a
   base path.
2. **Standalone** against an upstream Unity Catalog service. The `bin` feature
   builds the `files-api` binary (CLI `serve` / `healthcheck`, a self-contained
   auth layer, and YAML config), vending volume credentials through the upstream
   UC's `temporary-table-credentials` endpoint.

The `portal.files.v1` wire contract itself lives in the shared
[`olai-uc-files-proto`](../files-proto) crate (generated code); this crate only
implements it.

## Store backends

- **Unity Catalog volumes** (`client-arm`): the real backend. Each operation
  vends a scoped credential for the volume root via `UnityObjectStoreFactory` and
  runs against the relative path. Uploads stream through a 5 MiB multipart writer;
  downloads and recursive listings are lazy `object_store` streams.
- **In-memory** (`testing`): a process-local `BTreeMap` store for dependency-free
  local runs and tests.

Local (`/home`) and the `/home`-vs-`/Volumes` routing store from the sibling
`hydrofoil` implementation are intentionally deferred.

## Streaming semantics

`UploadFile` is a client stream (chunks in) and `DownloadFile` is a server stream
(chunks out); neither ever buffers the whole file. `ListDirectoryContents` is
unary + paged (a bounded page, for a UI); `ListDirectoryStream` is a lazy server
stream that scales to large directories.

## Running standalone

```bash
files-api serve --upstream-url https://<uc-host>/api/2.1/unity-catalog/ \
  --upstream-token "$UC_TOKEN" --backend unity
# operational endpoints (outside auth): /health /version /capabilities
files-api healthcheck   # used by the container HEALTHCHECK
```

See `--help` for the full flag set and the YAML config schema.
