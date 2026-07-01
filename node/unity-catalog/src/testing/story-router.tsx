// A tiny TanStack Router harness for stories whose components read route state
// (the Catalog Explorer keeps its selection in the `/catalog` route's `sel`
// search param via useSearch/useNavigate). The global preview decorator
// deliberately omits the router (most components don't need it), so components
// that DO need one wrap themselves with this.
//
// The route id must match what the component reads: useCatalogSelection() is
// pinned to `from: "/catalog"`, so the route path here is "catalog".

import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import type { ReactNode } from "react";

/** Mount `children` under a memory router with a single `/catalog` route that
 *  validates the `sel` search param, so route-aware catalog components work in
 *  isolation. `initialSel` seeds a selection (encoded `kind:fullName`). */
export function CatalogRouterHarness({
  children,
  initialSel,
}: {
  children: ReactNode;
  initialSel?: string;
}) {
  const rootRoute = createRootRoute();
  const catalogRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "catalog",
    validateSearch: (search: Record<string, unknown>) => ({
      sel: typeof search.sel === "string" ? search.sel : undefined,
    }),
    component: () => <>{children}</>,
  });

  const router = createRouter({
    routeTree: rootRoute.addChildren([catalogRoute]),
    history: createMemoryHistory({
      initialEntries: [
        initialSel
          ? `/catalog?sel=${encodeURIComponent(initialSel)}`
          : "/catalog",
      ],
    }),
  });

  // The router type here is local to the story harness; cast to satisfy the
  // RouterProvider generic without registering it globally.
  // biome-ignore lint/suspicious/noExplicitAny: story-local router instance
  return <RouterProvider router={router as any} />;
}
