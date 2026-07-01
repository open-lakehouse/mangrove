import {
  ThemeProvider,
  Toaster,
  TooltipProvider,
} from "@open-lakehouse/ui-kit";
import { EnvironmentScopeProvider } from "@open-lakehouse/unity-catalog";
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

const queryClient = new QueryClient();
const router = createAppRouter(queryClient);

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Root element #root not found");

createRoot(rootElement).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <UnityCatalogProvider client={ucClient}>
        {/* Single-environment app: one stable scope namespace for the UC tree. */}
        <EnvironmentScopeProvider scopeId="uc">
          <ThemeProvider>
            <TooltipProvider delayDuration={300}>
              <RouterProvider router={router} />
              <Toaster position="bottom-right" />
            </TooltipProvider>
          </ThemeProvider>
        </EnvironmentScopeProvider>
      </UnityCatalogProvider>
    </QueryClientProvider>
  </StrictMode>,
);
