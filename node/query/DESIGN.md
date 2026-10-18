# Design contract — `@open-lakehouse/query`

The query-preview **seam**: it turns "preview this table" into streamed Arrow IPC
for `@open-lakehouse/data-grid`. It ships no query engine and no UI. The host
registers a `QueryRunner` (the in-browser wasm engine, an RPC, a desktop IPC
call…), and the hook drives it. A **leaf**: it never imports Unity Catalog.

## Install & peers

Dependencies: `@open-lakehouse/data-grid`, `@bufbuild/protobuf`. Peers: `react` /
`react-dom` 19. No styles, so there's no Tailwind setup for this package itself;
render results with data-grid and follow its host setup.

## Seams

| Layer | Swap point | Contract |
| --- | --- | --- |
| Engine | `registerQueryRunner(runner)` | `(req: { sql, … }, { signal }) => AsyncIterable<{ arrowIpc, numRows }>`; types generated from `proto/query` (`RunQueryRequest` / `RunQueryResponse`) |
| Service | `<QueryServiceProvider service={…}>` / `setDefaultQueryService` | table-oriented `QueryService` built on the runner (`createQueryService`) |

Until a runner is registered, `hasQueryRunner()` is `false` and running throws
`NoQueryRunnerError`. **Gate preview UI on `hasQueryRunner()`** so a build without
an engine never shows a control that can only fail. `queryRunnerSupports(…)`
reports the runner's capabilities, and `createFallbackQueryRunner` chains runners.

## Usage

```tsx
import { DataGrid } from "@open-lakehouse/data-grid";
import { hasQueryRunner, registerQueryRunner, usePreview } from "@open-lakehouse/query";

// once, at startup
registerQueryRunner(async function* (req, { signal }) {
  for await (const batch of myEngine.run(req.sql, { signal })) {
    yield { arrowIpc: batch.ipc, numRows: batch.rows };
  }
});

function TablePreview({ table }: { table: string }) {
  const { store, version, running, error } = usePreview({ tableFullName: table, limit: 100 });
  if (error) return <p>{error.message}</p>;
  return <DataGrid store={store} version={version} running={running} />;
}

{hasQueryRunner() && <TablePreview table="main.default.trips" />}
```

## Rules

- **Do** register exactly one runner per app at startup; components read it
  through the service, never directly.
- **Do** honor the `signal`: previews are cancelled when the component unmounts
  or its inputs change.
- **Don't** hand-edit `src/gen/**`; regenerate from `proto/query` with
  `just generate-query-contract`.
- **Don't** add an engine implementation here; engines live in their own
  packages (e.g. `@open-lakehouse/query-wasm`) and plug in through the runner.
