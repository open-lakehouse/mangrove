# `@open-lakehouse/unity-catalog`

The Unity Catalog feature package: the catalog browser, per-entity detail panes,
the create/edit/delete and storage dialogs, the schema-driven form renderer, the
metastore storage admin table, and the React-Query data layer that talks to the
Unity Catalog REST API.

## Public surface

Import only from the package root — `@open-lakehouse/unity-catalog`:

| Export | Used by |
| --- | --- |
| `UnityCatalogProvider`, `useUnityCatalog` | app root (client injection) |
| `createUnityCatalogClient`, `defaultUnityCatalogClient`, `fetchClient`, `setDefaultUnityCatalogFetch` | app root (transport wiring) |
| `EnvironmentScopeProvider` | app root (feeds the env scope id) |
| `CatalogExplorer` | `routes/catalog.lazy` (route-level browser) |
| `CatalogDialogsProvider` | `environment/manager/EnvironmentManager` |
| `StorageTable`, `StorageLocationPicker`, type `StorageKind` | `environment/manager/EnvironmentDetail` (StorageTable); StorageLocationPicker is exported for future consumers |
| `Meta`, `MetaGrid` | `environment/manager/EnvironmentDetail` |
| `ListStates`, `TreeRow` | `editor/fileTree/FileTree` |
| `useCatalogs`, `useSchemas`, `useVolumes`, `useCredentials`, `useExternalLocations`, `prefetchCatalogs` | `editor/AddVolumeDialog`, `routes/import.lazy`, `routeTree` |
| `invalidateTables` | `routes/import.lazy` |
| `parseUcError` | `EnvironmentGate`, `ErrorBoundary` |

Everything else — the tree internals (`CatalogTree`, `RowMenu`, `DetailPane`,
`selection`, `ExpansionContext`, `groups`, `dialog-types`), the detail panes
under `detail/`, the dialog wiring (`dialogs`, the `*EntityDialog`s, the storage
dialogs under `storage/`), the form renderer under `forms/`, and the `uc/` data
layer internals — is package-internal; the `exports` map reaches only the root
barrel.

## Client injection

`uc/queries.ts` and `uc/mutations.ts` read their client from `useUnityCatalog()`
(provided by `UnityCatalogProvider`) rather than a module singleton, so the host
decides base URL / transport / auth. This is the seam a future proto-generated
WASM client swaps into with no hook changes (see
`docs/portable-uc-components.md`).

Every fetch — list reads (`useCatalogs`, …), mutations, AND detail reads
(`useCatalogDetail`, … and the storage dialogs) — routes through the injected
client. The `*DetailQuery(id)` functions and `prefetch*` helpers deliberately
bind the *default* client because they only derive query keys / warm caches;
key derivation is client-independent, so the keys they produce match the
injected-client hooks and caches stay aligned. `mutations.ts` reads
`*DetailQuery(id).queryKey` for `setQueryData` / `removeQueries` — a key, not a
fetch — which is why those stay on the default client.

The default client's transport is a stable indirection over a mutable fetch: the
host calls `setDefaultUnityCatalogFetch(fetch)` once at startup to route the
default client (and the back-compat `$api` / `fetchClient` singletons and the
prefetch helpers) through its own fetch — e.g. the Tauri desktop shell's IPC
fetch — without this package depending on the host's transport registry. Absent
that call, the default is the platform `fetch`.

## External dependencies

The package depends only on shared packages and one host edge:

- `@open-lakehouse/ui-kit` — the shared shadcn primitives and `cn`.
- Its own generated UC OpenAPI types — `src/uc-types.ts` re-exports the
  `openapi-typescript` output (`src/uc-api.d.ts`, generated from
  `openapi/unity-catalog.yaml` via `npm run gen:api`) that the data layer is typed
  against. (This absorbs what used to be a separate `@open-lakehouse/uc-client`
  package.) Note: this OSS-shaped UC spec differs from mangrove's native
  `openapi/openapi.yaml`; reconciling the two is deferred follow-up work.
- `@rjsf/*` — the JSON-schema form renderer backing `forms/SchemaForm`. The UC
  entity/storage form schemas live in `forms/schemas/` (generated from this repo's
  own UC proto by `scripts/gen-form-schemas.mjs`, i.e. `npm run gen:form-schemas`).
- **`./env-seam` — the single host edge.** `ExpansionContext` namespaces its
  persisted tree-expansion state per active environment, so it needs the current
  environment id. The package owns an `EnvironmentScopeProvider`; the host mounts
  it with its active-environment id. Absent a provider, the scope falls back to a
  stable constant (single namespace), so tests and stories render unwrapped. To
  embed the package elsewhere, mount `EnvironmentScopeProvider` with whatever id
  represents the embedder's current scope — nothing in core changes.

## Boundary guarantee

The package `exports` map reaches only `src/index.ts`, so external code can
import only the barrel — the "barrel-only" boundary is now native to the package
rather than enforced by a lint rule.

## Scope notes

- `StorageTable` is UC-specific — it hand-rolls its own `<table>` bound to UC
  types/hooks/detail panes. It is **not** the reusable `@open-lakehouse/data-grid`
  primitive and does not use it. Porting it onto a generic table is a separate,
  behavior-changing refactor (see the strategy doc).
- Dependency direction: this package may build on `@open-lakehouse/data-grid` in
  the future; `data-grid` never builds on it.
- `selection.ts` / `ExpansionContext` read TanStack Router and `sessionStorage`
  directly; making them prop/callback-driven is a separate headless-ification
  tracked in the strategy doc.
