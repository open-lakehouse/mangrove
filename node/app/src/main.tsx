import { LogQueryServiceProvider } from "@open-lakehouse/delta-log";
import { registerStubLogPreview } from "@open-lakehouse/delta-log/testing";
import { registerCatalogProvider } from "@open-lakehouse/editor";
import { QueryServiceProvider } from "@open-lakehouse/query-runner";
import {
  ThemeProvider,
  Toaster,
  TooltipProvider,
} from "@open-lakehouse/ui-kit";
import {
  createUnityCatalogClient,
  UnityCatalogProvider,
} from "@open-lakehouse/unity-catalog-client/react";
import {
  EnvironmentScopeProvider,
  ucCatalogProvider,
} from "@open-lakehouse/unity-catalog-ui";
import { FilesServiceProvider } from "@open-lakehouse/volume-files";
import { registerStubFiles } from "@open-lakehouse/volume-files/testing";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createAppRouter } from "./router";
import "./globals.css";

// The UC REST API is served on the SAME origin as this SPA by the `uc-server`
// binary (it serves both the API and the built bundle from `web/`). Point the
// client at the same-origin API root; the base URL is resolved relative to
// `window.location.origin`, so no host/port is hardcoded. In dev the Vite proxy
// forwards `/api` to a locally-running UC server (see vite.config.ts).
const ucClient = createUnityCatalogClient({
  baseUrl: `${window.location.origin}/api/2.1/unity-catalog`,
});

// The Delta-log tab (in TableDetail) is gated on hasLogQueryRunner(). This build
// registers the dev fixture stub runner, which renders a canned reconciled-log
// dataset end-to-end. A host lights up real data by registering a runner via
// registerLogQueryRunner (see @open-lakehouse/delta-log); the in-browser wasm
// engine that did so lives in open-lakehouse/hydrofoil (@open-lakehouse/query-wasm).
registerStubLogPreview();

// The volume file editor's SQL completion sources catalog/schema/table/column
// names from Unity Catalog. Register the UC-backed provider (the editor package
// itself is UC-agnostic — it ships an empty default and degrades to keyword-only
// completion until a provider is registered here). Late-bound, so ordering vs
// the editor's own bootstrap doesn't matter.
registerCatalogProvider(ucCatalogProvider);

// The volume Files tab (in VolumeDetail) is gated on hasFilesRunner(). This build
// registers the dev fixture stub runner, which renders a canned volume file tree
// end-to-end; a host registers a real runner via registerFilesRunner (see
// @open-lakehouse/volume-files). The stub is read-only (canWrite() is false), so
// the editor session opens files read-only.
registerStubFiles();

const queryClient = new QueryClient();
const router = createAppRouter(queryClient);

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element #root not found");

createRoot(rootElement).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <UnityCatalogProvider client={ucClient}>
        {/* The preview seam. This standalone build registers NO query runner, so
            the default service's runner throws and TableDetail keeps its preview
            gated off (see @open-lakehouse/query-runner). A host registers a runner
            via registerQueryRunner to light it up. */}
        <QueryServiceProvider>
          {/* The Delta-log seam, beside the preview seam. The stub runner above
              backs it in this build; a host swaps in a real runner via
              registerLogQueryRunner (see @open-lakehouse/delta-log). */}
          <LogQueryServiceProvider>
            {/* The volume-files seam, backing the VolumeDetail Files tab. The
                stub runner registered above serves it; the editor reads
                files through this service. */}
            <FilesServiceProvider>
              {/* Single-environment app: one stable scope namespace for the UC tree. */}
              <EnvironmentScopeProvider scopeId="uc">
                <ThemeProvider>
                  <TooltipProvider delayDuration={300}>
                    <RouterProvider router={router} />
                    <Toaster position="bottom-right" />
                  </TooltipProvider>
                </ThemeProvider>
              </EnvironmentScopeProvider>
            </FilesServiceProvider>
          </LogQueryServiceProvider>
        </QueryServiceProvider>
      </UnityCatalogProvider>
    </QueryClientProvider>
  </StrictMode>,
);
