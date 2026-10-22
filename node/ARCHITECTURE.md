# UI architecture — state, seams, and composition

This is the target architecture for the `@open-lakehouse/*` UI packages as a
composable framework: the Unity Catalog views in this repo, the lineage views
in [headwaters](https://github.com/open-lakehouse/headwaters), hosts like
[hydrofoil](https://github.com/open-lakehouse/hydrofoil), and the links between
them (a table's lineage, a lineage node's catalog entry).

It records where we are, what we decided to change, and what we decided *not*
to build. Per-package contracts stay in each package's `DESIGN.md`; the package
map and dependency rules stay in [`README.md`](README.md).

## Where we are

| | mangrove `node/` | hydrofoil `node/ui` | headwaters `node/` |
|---|---|---|---|
| Server state | TanStack Query through `openapi-react-query`; keys derive from shared init builders in [`unity-catalog-client/src/uc/queries.ts`](unity-catalog-client/src/uc/queries.ts), mutations invalidate by predicate | TanStack Query with tuned defaults (`lib/query-client.ts`) | TanStack Query, root key `"headwaters-read"` |
| `QueryClient` config | bare `new QueryClient()` in [`app/src/main.tsx`](app/src/main.tsx) — `staleTime: 0`, refetch on mount and focus | `staleTime` 60s, `gcTime` 5m, no focus refetch, `keepPreviousData` | `staleTime` 10s |
| Polling | none | none (environment health only) | none |
| Arrow results | `ArrowResultStore` (data-grid) driven by `PreviewRun` / `LogQueryRun` over `useSyncExternalStore` | its own `RunController` (per tab, rAF-coalesced) over the same `ArrowResultStore` | — |
| Seams | module-level `register*` / `has*` singletons: `QueryRunner`, `LogQueryRunner`, `FilesRunner`, `CatalogProvider` | its own copies of the query seam, catalog completion, Monaco setup and files store, plus host seams (environment, notebook, file picker) | one transport seam |
| UI primitives | `ui-kit` | `ui-kit`, `data-grid`, `unity-catalog-client` | own button, raw `hsl(var(--…))` |
| Cross-service | none; detail tabs hard-coded in [`detail/TableDetail.tsx`](unity-catalog-ui/src/detail/TableDetail.tsx), [`DetailPane.tsx`](unity-catalog-ui/src/DetailPane.tsx) is an if-chain per kind | iframes (MLflow, marimo) | none; datasets keyed by OpenLineage `{namespace, name}` |

The leaf packages (`data-grid`, `query`, `log-query`, `files`, `editor`) are in
good shape: UC-agnostic, abort-aware, with streaming stores behind
`useSyncExternalStore`. The problems sit at the edges: global seams,
hydrofoil's parallel copies, and logic shipped inside copy-in blocks.

## Decisions

### No unified lakehouse data store

We considered one store that holds catalog metadata, lineage, and query
results together. We are not building it.

- There is less to consolidate than it seems. Nothing polls today; every
  refresh is invalidation-driven.
- Catalog and lineage metadata are request/response server state. TanStack
  Query already handles them: dedup, gc, retries, Suspense, devtools. A custom
  store would rebuild that.
- Arrow results are a different kind of data: large, streamed, scoped to a
  run, and not refetchable by key. They need explicit lifetime (abort, dispose
  on tab close or environment switch), not cache eviction. Putting them in the
  query cache, or both in a third store, mixes two lifecycles that are easier
  to reason about apart.

So there are **two caches, on purpose**:

1. **One shared `QueryClient`** for all metadata services. Every service's keys
   start with a service namespace (`["uc", …]`, `["lineage", …]`), and every
   client package exports `queryOptions` factories next to its hooks.
   Combining data across services is a composite hook built from those
   factories (`useQueries`, dependent queries) — e.g. a table's lineage summary
   is a lineage query that depends on the UC table query.
2. **Run stores** for streamed Arrow data: `ArrowResultStore` plus a run
   controller that owns abort, render coalescing, and disposal.

The client should ship a `createLakehouseQueryClient()` preset with the
defaults hydrofoil already settled on (catalog metadata changes rarely; a
focus refetch wave on every tab switch costs more than it gains). Views that
need fresher data override per query.

List→detail seeding in `queries.ts` currently writes detail entries from a
`useEffect` in every list hook. That runs once per mounted consumer and stamps
the detail entry as fresh with the list payload. The detail queries should
read the list cache as `initialData` with `initialDataUpdatedAt` instead, so
freshness follows the list's real fetch time.

### Seams become a context, not module globals

Each seam is a module-level `let current` with `register*` / `has*`. This works
for one app with one backend, but:

- there can only be one runner per JS realm, which clashes with hydrofoil's
  multiple environments and makes tests share state;
- `has*()` is read during render and is not reactive, so a late registration
  doesn't re-render the gate;
- the same registry is written out four times here and again in hydrofoil.

The target is one `LakehouseServices` provider holding a typed service map
(UC client, query runner, log-query runner, files runner, catalog provider,
and any later service such as a lineage client) and a capabilities object.
Components read it through `useService(key)` / `useCapability(key)`. A host
with several environments mounts one provider per environment.

The existing `register*` functions stay as the default layer beneath the
context, so current hosts keep working unchanged and migrate when they want.

### One run controller, upstreamed from hydrofoil

Hydrofoil's `RunController` (abort on rerun, one render per animation frame,
lives as long as its tab, disposed with its environment through a session
registry) is the general version of what `query`'s `PreviewRun` and
`log-query`'s `LogQueryRun` do for previews. It moves into `@open-lakehouse/query`
as the run primitive; `usePreview` and `useLogPreview` are rebuilt on it.

With that in place hydrofoil drops its parallel copies and consumes `query`,
`editor` and `files` directly. That also brings it onto the editor's pinned
Monaco pair (see [`editor/DESIGN.md`](editor/DESIGN.md)) and off the old
`@open-lakehouse/unity-catalog` package name.

### Headless hooks where copy-in distribution bites

The leaves are already headless; they don't get another layer.

`unity-catalog-ui` is different. It ships as shadcn blocks that hosts copy in,
and the blocks carry real logic: tree expansion
([`ExpansionContext.tsx`](unity-catalog-ui/src/ExpansionContext.tsx)),
URL-backed selection ([`selection.ts`](unity-catalog-ui/src/selection.ts),
which also hard-codes the host's `/catalog` route), paging, create-reconcile,
and tab gating. Every host that copies the blocks copies that logic too, and
the copies drift.

That logic moves into a published headless package,
`@open-lakehouse/unity-catalog-react` — `useCatalogTree`, `useCatalogSelection`
(with the URL binding supplied by the host), `useEntityDetail`, `useDetailTabs`
— in the same split TanStack Table uses. The blocks become thin JSX over those
hooks, so what hosts copy is presentation they are expected to own.

### Detail views take contributed extensions

Today a new tab on a table means editing `TableDetail`. Instead, a detail view
renders contributions registered through `LakehouseServices`:

```ts
interface DetailExtension<E> {
  id: string;
  kind: SelectableKind;              // "table", "volume", …
  label: string;
  when(entity: E, caps: Capabilities): boolean;
  render(entity: E): ReactNode;
}
```

Preview, Delta log, and Files become built-in contributions. Lineage is one
more, shipped by headwaters' `lineage-ui`. Neither repo depends on the other's
UI: mangrove knows the extension type, headwaters implements it, and the host
registers both. Biome `noRestrictedImports` rules keep `lineage-ui` and the
`unity-catalog-*` packages from importing each other.

The same goes for headwaters' primitives: it adopts `ui-kit` and the token
contract in [`ui-kit/DESIGN.md`](ui-kit/DESIGN.md) instead of its own button
and raw CSS variables.

## Open question: dataset identity

Linking a UC table to its lineage needs both sides to agree on what a dataset
*is*. Headwaters keys datasets by OpenLineage `{namespace, name}`; UC uses
`catalog.schema.table`. Nothing maps one to the other yet. Options:

- **Headwaters stores the UC name.** Ingest records the UC full name (column or
  facet) and the read API can look a dataset up by it. Exact, but needs a
  headwaters proto change and producers that emit the name.
- **Client-side resolver.** The UI derives the OpenLineage identity from the
  table's storage location (or the reverse). No server change, but fragile
  for managed tables and anything that reports a different namespace.

Either way the UI side is a shared `DatasetRef` type and a resolver behind the
lineage service in `LakehouseServices`, so the choice can change later without
touching the views. This is undecided; the lineage extension waits on it.

## Sequencing

Each step stands on its own; stopping after step 3 still removes the
duplication.

1. **Query client hygiene** (#217) — `createLakehouseQueryClient()` preset, service-
   namespaced keys, `initialData`-based list→detail seeding.
2. **`LakehouseServices` context** (#218) — service map + capabilities, `register*`
   kept as the fallback layer.
3. **Upstream the run controller** (#219) — move it into `query`, rebuild the preview
   hooks on it, migrate hydrofoil onto `query` / `editor` / `files`.
4. **Headless UC hooks** (#220) — extract `unity-catalog-react`, slim the blocks.
5. **Detail extensions and lineage** (#221) — extension slots, then the lineage
   contribution once dataset identity is settled.
