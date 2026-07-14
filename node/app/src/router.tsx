import { CatalogExplorer } from "@open-lakehouse/unity-catalog";
import { prefetchCatalogs } from "@open-lakehouse/unity-catalog-client";
import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";
import { AppShell } from "./AppShell";

export interface RouterContext {
  queryClient: QueryClient;
}

// The root renders the app shell (global header + theme control) around the
// routed content via its own <Outlet />.
const rootRoute = createRootRouteWithContext<RouterContext>()({
  component: AppShell,
});

// `/` has no content of its own — send the user straight to the catalog browser.
const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/catalog" });
  },
});

interface CatalogSearch {
  // Selected object, encoded as `kind:fullName`. CatalogExplorer reads this via
  // useCatalogSelection(), which is pinned to `from: "/catalog"` — so the route
  // path and the `sel` search param must match exactly.
  sel?: string;
}

const catalogRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "catalog",
  // Selection is URL-addressable so detail views are deep-linkable.
  validateSearch: (search: Record<string, unknown>): CatalogSearch => ({
    sel: typeof search.sel === "string" ? search.sel : undefined,
  }),
  // Warm the catalog list before the route component mounts.
  loader: ({ context }) => prefetchCatalogs(context.queryClient),
  component: CatalogExplorer,
});

const routeTree = rootRoute.addChildren([indexRoute, catalogRoute]);

export function createAppRouter(queryClient: QueryClient) {
  return createRouter({
    routeTree,
    context: { queryClient },
    defaultPreload: "intent",
  });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
