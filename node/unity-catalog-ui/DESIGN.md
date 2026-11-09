# Design contract — `@open-lakehouse/unity-catalog-ui`

The Unity Catalog browser: catalog explorer (tree + detail panes for catalogs,
schemas, tables, volumes, functions, models), create/edit/delete dialogs,
storage credentials and external locations, the volume file browser and editor,
the table preview, and the Delta-log tab. **Presentation only**: all data access
comes from `@open-lakehouse/unity-catalog-client/react`. This is the composed
package; it builds on every leaf (ui-kit, arrow-grid, editor, query-runner,
volume-files, delta-log), and nothing builds on it.

## Distribution

An npm package on GitHub Packages, like the leaves it builds on, so a host such
as the luminescent-lakehouse desktop shell installs it and mounts the explorer
as is. It takes the leaves that carry a seam (`unity-catalog-client`, `query-runner`,
`volume-files`, `delta-log`, `editor`) as peers: their providers and registries are
module state, so the host and this package must share one copy. Inside this
workspace it is consumed from `src/` through the `source` export condition.

## Peers

`react` / `react-dom` 19, `@tanstack/react-query` 5, `@tanstack/react-router`
1, `@open-lakehouse/host` and `@open-lakehouse/ui-kit` (trestle), and the seam
packages above. The host must provide exactly one copy of each (dedupe in the
bundler).

## Host setup

1. **Providers.** Mount, outside the routes that render these components:
   - `QueryClientProvider` (TanStack Query);
   - `UnityCatalogProvider` from `@open-lakehouse/unity-catalog-client/react`:
     without a `client`, it takes the transport from the `@open-lakehouse/host`
     host's `unity-catalog` binding; a host without the contract passes a client
     built by `createUnityCatalogClient({ baseUrl })`;
   - `QueryServiceProvider`, `LogQueryServiceProvider` and `FilesServiceProvider`
     (optional: each falls back to its package's default service);
   - ui-kit's `ThemeProvider` and `TooltipProvider`, and a `Toaster` for the
     dialogs' success/error notifications;
   - `EnvironmentScopeProvider`, which namespaces persisted tree state per
     environment (optional: without `scopeId` it uses the host's environment
     key, and without a host one shared namespace).
2. **Router.** `CatalogExplorer` reads and writes the selection through TanStack
   Router search params, so render it inside a route of a `RouterProvider`.
3. **Optional capabilities.** Previews, the Delta-log tab and the volume file
   browser only light up once the host registers their runners
   (`registerQueryRunner`, `registerLogQueryRunner`, `registerFilesRunner`). SQL
   completion in the volume editor needs `registerCatalogProvider(ucCatalogProvider)`.
4. **Tailwind.** Define the ui-kit tokens ([ui-kit's `DESIGN.md`](https://github.com/open-lakehouse/trestle/blob/ui-kit-v0.0.1/node/ui-kit/DESIGN.md))
   plus the `--action-*` palette. The copied block files live in the host, so its
   Tailwind build scans them already; scan **every npm package they render**:

   ```css
   @source "../node_modules/@open-lakehouse/ui-kit/dist";
   @source "../node_modules/@open-lakehouse/arrow-grid/dist";
   @source "../node_modules/@open-lakehouse/editor/dist";
   @source "../node_modules/@open-lakehouse/delta-log/dist";
   ```

   [`node/app`](../app/) is the reference host for all of the above
   (`src/main.tsx`, `src/globals.css`, `src/router.tsx`).

## Public surface

| Export | Purpose |
| --- | --- |
| `CatalogExplorer` | the full browser (tree + detail + dialogs); route-level |
| `ExternalDataPage`, `StorageTable`, `StorageLocationPicker`, `StorageKind` | credentials / external locations |
| `CatalogDialogsProvider` | create/edit/delete dialogs for hosts composing their own layout |
| `Meta`, `MetaGrid`, `TreeRow`, `ListStates` | building blocks reused by hosts |
| `EnvironmentScopeProvider` | the environment-scope seam |
| `ucCatalogProvider` | the editor's `CatalogProvider` backed by Unity Catalog |

## Usage

In this workspace the components import from `@open-lakehouse/unity-catalog-ui`;
a host that added the block imports them from wherever `shadcn add` put them.

```tsx
import { createUnityCatalogClient, UnityCatalogProvider } from "@open-lakehouse/unity-catalog-client/react";
import { CatalogExplorer } from "@open-lakehouse/unity-catalog-ui"; // or the block's local path
import { ThemeProvider, TooltipProvider } from "@open-lakehouse/ui-kit";

const uc = createUnityCatalogClient({ baseUrl: "/api/2.1/unity-catalog" });

<QueryClientProvider client={queryClient}>
  <UnityCatalogProvider client={uc}>
    <ThemeProvider>
      <TooltipProvider>
        <RouterProvider router={router} /> {/* a route renders <CatalogExplorer /> */}
      </TooltipProvider>
    </ThemeProvider>
  </UnityCatalogProvider>
</QueryClientProvider>;
```

## Rules

- **Do** fetch through the client package's hooks. Components never call
  `fetch` and never build their own UC client.
- **Do** regenerate the form schemas (`bun run gen:form-schemas`) after UC proto
  changes rather than editing `src/forms/schemas/*.json`.
- **Don't** import this package from a leaf; leaves stay UC-agnostic.
