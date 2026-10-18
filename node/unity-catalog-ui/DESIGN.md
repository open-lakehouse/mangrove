# Design contract — `@open-lakehouse/unity-catalog-ui`

The Unity Catalog browser: catalog explorer (tree + detail panes for catalogs,
schemas, tables, volumes, functions, models), create/edit/delete dialogs,
storage credentials and external locations, the volume file browser and editor,
the table preview, and the Delta-log tab. **Presentation only**: all data access
comes from `@open-lakehouse/unity-catalog-client/react`. This is the composed
package; it builds on every leaf (ui-kit, data-grid, editor, query, files,
log-query), and nothing builds on it.

## Install & peers

Peers: `react` / `react-dom` 19, `@tanstack/react-query` 5,
`@tanstack/react-router` 1. The host must provide exactly one copy of each
(dedupe in the bundler). The leaf packages come in as dependencies.

## Host setup

1. **Providers.** Mount, outside the routes that render these components:
   - `QueryClientProvider` (TanStack Query);
   - `UnityCatalogProvider` from `@open-lakehouse/unity-catalog-client/react`,
     with a client built by `createUnityCatalogClient({ baseUrl })`;
   - `QueryServiceProvider`, `LogQueryServiceProvider` and `FilesServiceProvider`
     (optional: each falls back to its package's default service);
   - ui-kit's `ThemeProvider` and `TooltipProvider`, and a `Toaster` for the
     dialogs' success/error notifications;
   - `EnvironmentScopeProvider scopeId=…`, which namespaces persisted tree state
     per environment (optional; defaults to one namespace).
2. **Router.** `CatalogExplorer` reads and writes the selection through TanStack
   Router search params, so render it inside a route of a `RouterProvider`.
3. **Optional capabilities.** Previews, the Delta-log tab and the volume file
   browser only light up once the host registers their runners
   (`registerQueryRunner`, `registerLogQueryRunner`, `registerFilesRunner`). SQL
   completion in the volume editor needs `registerCatalogProvider(ucCatalogProvider)`.
4. **Tailwind.** Define the ui-kit tokens ([`ui-kit/DESIGN.md`](../ui-kit/DESIGN.md))
   plus the `--action-*` palette, and scan this package **and every package it
   renders**:

   ```css
   @source "../node_modules/@open-lakehouse/unity-catalog-ui/dist";
   @source "../node_modules/@open-lakehouse/ui-kit/dist";
   @source "../node_modules/@open-lakehouse/data-grid/dist";
   @source "../node_modules/@open-lakehouse/editor/dist";
   @source "../node_modules/@open-lakehouse/log-query/dist";
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

```tsx
import { createUnityCatalogClient, UnityCatalogProvider } from "@open-lakehouse/unity-catalog-client/react";
import { CatalogExplorer } from "@open-lakehouse/unity-catalog-ui";
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
