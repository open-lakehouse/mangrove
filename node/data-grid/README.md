# `@open-lakehouse/data-grid`

The reusable, virtualized, Arrow-backed table primitive. A **leaf** package: a
lower-level building block other features depend on, which itself depends on
nothing application-specific (only Arrow, TanStack, and the shared
`@open-lakehouse/ui-kit`).

## Public surface

Import only from the package root — `@open-lakehouse/data-grid`:

| Export | Kind | Purpose |
| --- | --- | --- |
| `DataGrid` | component | Virtualized, sortable grid that renders an `ArrowResultStore`. Props: `{ store, version, running, className? }`. |
| `ArrowResultStore` | class | The grid's input contract. Consumers build one (from Arrow IPC bytes, query results, or fixtures) and pass it as `store`. |
| `ArrowStoreInfo` | type | Cheap read-only summary of a store's contents. |

Everything else — `data-grid-cell`, `data-grid-header`, the Arrow-support
modules under `lib/` (`arrowResultStore`, `useArrowTable`, `cellFormatters`,
`arrowTypeLabel`, `sortValues`), and `story-fixtures` — is internal; the package
`exports` map only exposes the root barrel, so it cannot be imported from
outside.

## Consumers

- `@open-lakehouse/unity-catalog`: the upcoming **table-preview** surface renders
  query results through `<DataGrid>`. It builds an `ArrowResultStore` from the
  data it fetches and hands it to the grid.
- hydrofoil's `@open-lakehouse/ui` (SQL editor result pane, import preview) also
  consumes this package via the sibling-repo `file:` link.

Note the direction in every case: consumers build the store and depend on this
package; the package never depends on them, which is why it ships its own
`story-fixtures` for its own stories.

## The `ArrowResultStore` seam (future query layer)

`ArrowResultStore` is deliberately the grid's **only** input contract, which
makes it the injection point for whatever produces the data. Today consumers
build a store directly from Arrow IPC bytes or fixtures. The planned
data-fetch/processing layer (a future `@open-lakehouse/query`-style package, likely
wasm-backed and built against mangrove's Rust) will produce an `ArrowResultStore`
too — so it slots in **without any change to this package**. Keeping the grid
agnostic of *how* the store is produced is the invariant; do not add
query/transport concerns here.

## Boundary rule (enforced)

`data-grid` is a leaf: a Biome `noRestrictedImports` rule in `node/biome.json`
forbids it from importing `@open-lakehouse/unity-catalog`. The dependency
direction is the invariant: the Unity Catalog package may build on `data-grid`;
`data-grid` never builds on it. The "barrel-only" guarantee is native — the
package `exports` map reaches only `src/index.ts`.

## Scope note

The grid is intentionally Arrow-specific — its input is `ArrowResultStore`.
Generalizing it beyond Arrow is a separate, behavior-changing refactor, not part
of this package's contract.
