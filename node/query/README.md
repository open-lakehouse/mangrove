# @open-lakehouse/query

The in-browser query seam for the Unity Catalog UI. It turns a table reference
into streamed Arrow IPC that feeds `@open-lakehouse/data-grid`'s
`ArrowResultStore` — so `TableDetail` can preview rows (`SELECT * FROM t LIMIT
100`) without a dedicated server-side query service.

This package is a **leaf**: it depends on `@open-lakehouse/data-grid` and React,
never on `@open-lakehouse/unity-catalog` (enforced by a Biome rule in
`node/biome.json`). Unity Catalog builds on it; never the reverse.

## It ships no runtime query implementation — on purpose

The whole point of the in-browser query layer is a default the website can ship
needing only a running Unity Catalog and **no extra service**. That default is
the in-browser wasm engine (a later phase), which isn't landed yet. So this
package provides the **seam**, not an engine:

- Until a runner is registered, the low-level `queryRunner` throws
  `NoQueryRunnerError`, and `hasQueryRunner()` returns `false`.
- The UI gates its preview affordance off `hasQueryRunner()` (plus a feature
  flag), so the standalone build never shows a preview that can only error.

A host or a later phase installs a real runner:

```ts
import { registerQueryRunner } from "@open-lakehouse/query";
registerQueryRunner(async function* (req, { signal }) {
  // execute req.sql however you like (wasm engine, an RPC, a Tauri invoke)…
  yield { arrowIpc, numRows };
});
```

This is the same seam hydrofoil's UI uses (`node/ui/src/lib/query/runner.ts`);
mangrove owns the contract, downstream implements it.

## Two layers

- **`QueryRunner`** (`runner.ts`) — the low-level swap point. `sql` in,
  `AsyncIterable<{ arrowIpc, numRows }>` out. Typed against the generated
  `open_lakehouse.query.v1` contract (`proto/query`). Swap it with
  `registerQueryRunner`.
- **`QueryService` / `usePreview` / `QueryServiceProvider`** (`api.ts`,
  `context.tsx`, `usePreview.ts`) — the table-oriented surface the UI consumes.
  `usePreview({ tableFullName, limit })` builds preview SQL, drives the runner,
  and streams chunks into an `ArrowResultStore`; render with
  `<DataGrid store version running />`.

## The contract

`src/gen/**` is generated from `proto/query/open_lakehouse/query/v1/svc.proto`
(mangrove-owned) via `just generate-query-contract`. It generates **TypeScript
message types only** — no transport client; the runner is registered at runtime.
Do not hand-edit generated files.
