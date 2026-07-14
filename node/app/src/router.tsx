import {
  CatalogExplorer,
  ExternalDataPage,
  type StorageKind,
} from "@open-lakehouse/unity-catalog";
import { prefetchCatalogs } from "@open-lakehouse/unity-catalog-client";
import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  redirect,
  useNavigate,
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
  // For a selected schema, which child-kind tab is active (table/volume/...).
  // Validated to a known kind inside the package; carried through here as a
  // string, mirroring `sel`.
  tab?: string;
}

const catalogRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "catalog",
  // Selection is URL-addressable so detail views are deep-linkable.
  validateSearch: (search: Record<string, unknown>): CatalogSearch => ({
    sel: typeof search.sel === "string" ? search.sel : undefined,
    tab: typeof search.tab === "string" ? search.tab : undefined,
  }),
  // Warm the catalog list before the route component mounts.
  loader: ({ context }) => prefetchCatalogs(context.queryClient),
  component: CatalogExplorer,
});

// Metastore-level storage admin ("External Data"). The active securable kind is
// URL-addressable so the sidebar cog can deep-link straight to a tab, and the
// tab is shareable / survives reload.
interface ExternalDataSearch {
  kind: StorageKind;
}

const STORAGE_KINDS: StorageKind[] = ["external_location", "credential"];

const externalDataRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "external-data",
  validateSearch: (search: Record<string, unknown>): ExternalDataSearch => ({
    kind: STORAGE_KINDS.includes(search.kind as StorageKind)
      ? (search.kind as StorageKind)
      : "external_location",
  }),
  component: ExternalDataRoute,
});

function ExternalDataRoute() {
  const { kind } = externalDataRoute.useSearch();
  const navigate = useNavigate();
  return (
    <ExternalDataPage
      kind={kind}
      onKindChange={(next) =>
        navigate({ to: "/external-data", search: { kind: next } })
      }
    />
  );
}

const routeTree = rootRoute.addChildren([
  indexRoute,
  catalogRoute,
  externalDataRoute,
]);

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
