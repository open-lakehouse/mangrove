import { registerCatalogProvider } from "@open-lakehouse/editor";
import { LogQueryServiceProvider } from "@open-lakehouse/log-query";
import { registerStubLogPreview } from "@open-lakehouse/log-query/testing";
import { QueryServiceProvider } from "@open-lakehouse/query";
import {
  registerWasmLogPreview,
  registerWasmPreview,
} from "@open-lakehouse/query-wasm";
import {
  ThemeProvider,
  Toaster,
  TooltipProvider,
} from "@open-lakehouse/ui-kit";
import {
  EnvironmentScopeProvider,
  ucCatalogProvider,
} from "@open-lakehouse/unity-catalog";
import {
  createUnityCatalogClient,
  UnityCatalogProvider,
} from "@open-lakehouse/unity-catalog-client";
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

// In wasm-enabled builds (VITE_ENABLE_WASM_QUERY=true + `just build-query-wasm`)
// this registers the in-browser engine as the app's query runner; default
// builds alias @open-lakehouse/query-wasm to a no-op stub (vite.config.ts), so
// no runner is registered and the preview UI stays gated off. The preview
// section itself is additionally flag-gated via VITE_ENABLE_PREVIEW.
registerWasmPreview({
  baseUrl: `${window.location.origin}/api/2.1/unity-catalog`,
});

// The Delta-log tab (in TableDetail) is gated on hasLogQueryRunner(); registering
// a runner here lights it up for Delta tables. In wasm-enabled builds we register
// the REAL in-browser log runner (reconciled files + reconciled action stream,
// backed by crates/query-wasm) — resolving through the same @open-lakehouse/query-wasm
// package the table preview uses. In default builds that package is aliased to a
// no-op stub, so we fall back to the dev fixture stub runner, which renders a
// canned reconciled-log dataset end-to-end without wasm.
if (import.meta.env.VITE_ENABLE_WASM_QUERY === "true") {
  registerWasmLogPreview({
    baseUrl: `${window.location.origin}/api/2.1/unity-catalog`,
  });
} else {
  registerStubLogPreview();
}

// The volume file editor's SQL completion sources catalog/schema/table/column
// names from Unity Catalog. Register the UC-backed provider (the editor package
// itself is UC-agnostic — it ships an empty default and degrades to keyword-only
// completion until a provider is registered here). Late-bound, so ordering vs
// the editor's own bootstrap doesn't matter.
registerCatalogProvider(ucCatalogProvider);

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
            gated off (see @open-lakehouse/query). A host or the future wasm engine
            registers a runner via registerQueryRunner to light it up. */}
        <QueryServiceProvider>
          {/* The Delta-log seam, beside the preview seam. The stub runner above
              backs it in this build; a host / the future wasm engine swaps in a
              real runner via registerLogQueryRunner (see @open-lakehouse/log-query). */}
          <LogQueryServiceProvider>
            {/* Single-environment app: one stable scope namespace for the UC tree. */}
            <EnvironmentScopeProvider scopeId="uc">
              <ThemeProvider>
                <TooltipProvider delayDuration={300}>
                  <RouterProvider router={router} />
                  <Toaster position="bottom-right" />
                </TooltipProvider>
              </ThemeProvider>
            </EnvironmentScopeProvider>
          </LogQueryServiceProvider>
        </QueryServiceProvider>
      </UnityCatalogProvider>
    </QueryClientProvider>
  </StrictMode>,
);
