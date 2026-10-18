# Design contract — `@open-lakehouse/data-grid`

A virtualized, sortable result grid that renders Apache Arrow data. It is a
**leaf**: it knows nothing about Unity Catalog, queries or transports. Whatever
produces the data hands it an `ArrowResultStore`.

## Install & peers

Dependencies: `@open-lakehouse/ui-kit`, `apache-arrow` 21, `@tanstack/react-table`,
`@tanstack/react-virtual`, `lucide-react`. Peers: `react` / `react-dom` 19 (one
deduped copy in the host).

## Host setup

The grid is styled with Tailwind utilities over the ui-kit token contract and
ships no CSS of its own. The host must:

1. Define the design tokens from [`ui-kit/DESIGN.md`](../ui-kit/DESIGN.md) on
   `:root` and `.dark`, and map them with `@theme inline`.
2. Let Tailwind generate this package's classes, plus ui-kit's:

   ```css
   /* paths are relative to the host stylesheet */
   @source "../node_modules/@open-lakehouse/data-grid/dist";
   @source "../node_modules/@open-lakehouse/ui-kit/dist";
   ```

3. Give the grid a bounded height (it virtualizes inside its own scroll box).

## The seam: `ArrowResultStore`

The store is the grid's only input. Build one from Arrow IPC bytes (query
results, a file, fixtures) and append as chunks stream in. `version` is the
re-render signal: bump it whenever the store changes. `running` shows the
streaming state.

## Usage

```tsx
import { ArrowResultStore, DataGrid } from "@open-lakehouse/data-grid";

const store = new ArrowResultStore();
store.append(arrowIpcBytes); // Uint8Array of an Arrow IPC stream

<div className="h-96">
  <DataGrid store={store} version={1} running={false} />
</div>;
```

For streamed query previews, `usePreview` from `@open-lakehouse/query` returns a
ready `{ store, version, running }` to spread into the grid.

The cell helpers (`formatCell`, `formatScalarText`), nested-type accessors
(`structChildren`, `structFieldByName`, `resolveChildPath`) and temporal helpers
(`timestampToEpochMs`, `toAxisNumber`, `isOrderableType`) are exported for hosts
that render Arrow values outside the grid with the same formatting.

## Rules

- **Do** keep the grid data-source agnostic: produce an `ArrowResultStore`, never
  teach the grid about queries, catalogs or fetch.
- **Do** pass a new `version` after every append; the grid re-reads on it.
- **Don't** import package internals (`lib/*`, cells, header); only the root
  barrel is exported.
- **Don't** expect non-Arrow inputs; generalizing beyond Arrow is out of scope.
