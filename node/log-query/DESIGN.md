# Design contract — `@open-lakehouse/log-query`

The Delta-log explorer: a log-query **seam** (a host-registered runner streams a
table's Delta log as Arrow) plus the views that render it, the reconciled
**actions log** and the **min/max boxes** of per-file column statistics. A
**leaf**: it never imports Unity Catalog. `@open-lakehouse/unity-catalog-ui` mounts
it as the Delta-log tab of a table.

## Install & peers

Dependencies: `@open-lakehouse/data-grid`, `@open-lakehouse/ui-kit`, `apache-arrow`
21, `@tanstack/react-virtual`, `lucide-react`. Peers: `react` / `react-dom` 19.

## Host setup

Styled with Tailwind utilities over the ui-kit tokens (see
[`ui-kit/DESIGN.md`](../ui-kit/DESIGN.md)), and it uses the categorical
`--action-*` palette the reference host defines (`node/app/src/globals.css`).
Scan this package and the packages it renders:

```css
@source "../node_modules/@open-lakehouse/log-query/dist";
@source "../node_modules/@open-lakehouse/data-grid/dist";
@source "../node_modules/@open-lakehouse/ui-kit/dist";
```

Skipping this `@source` is easy to miss: the views still render, but every class
only this package uses is missing.

## Seams

| Layer | Swap point | Contract |
| --- | --- | --- |
| Engine | `registerLogQueryRunner(runner)` | `(req: { target?, kind?, limit? }, { signal }) => AsyncIterable<LogQueryChunk>` (Arrow IPC); `kind` picks the log surface (default `reconciled`) |
| Service | `<LogQueryServiceProvider service={…}>` / `setDefaultLogQueryService` | `LogQueryService` built on the runner |

Gate the Delta-log UI on `hasLogQueryRunner()`. The in-browser implementation
is `@open-lakehouse/query-wasm`'s log runner; `@open-lakehouse/log-query/testing`
has a stub with fixtures.

## Usage

```tsx
import { ActionsLog, hasMinMaxAxes, MinMaxView, useLogPreview } from "@open-lakehouse/log-query";

function DeltaLog({ table }: { table: string }) {
  const { store, version, running, error } = useLogPreview({ target: table });
  if (error) return <p>{error.message}</p>;
  return (
    <>
      <ActionsLog store={store} version={version} running={running} />
      {hasMinMaxAxes(store.schema?.fields) && <MinMaxView store={store} version={version} />}
    </>
  );
}
```

## Rules

- **Do** treat `target` as opaque (a `catalog.schema.table` name or a storage
  path); the runner resolves it.
- **Do** keep the action palette in the host's `globals.css`, next to the other
  tokens, rather than hard-coding colors in components.
- **Don't** import Unity Catalog packages here; the UC wiring belongs in
  `unity-catalog-ui`.
