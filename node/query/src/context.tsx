// Query-service context — the injection point for a `QueryService`, mirroring
// unity-catalog-client's `UnityCatalogProvider` / `useUnityCatalog`.
//
// The UI reads its service from here via `useQueryService()` rather than
// importing a singleton, so a host decides which service (and thus which runner)
// is in effect by mounting `<QueryServiceProvider>`. When no provider is mounted,
// `useQueryService()` falls back to `defaultQueryService()`, so tests and
// Storybook stories need no wrapper.

import { createContext, type ReactNode, useContext } from "react";
import { defaultQueryService } from "./api";
import type { QueryService } from "./types";

const QueryServiceContext = createContext<QueryService | null>(null);

/**
 * Provide a {@link QueryService} to the preview UI. Mount once near the app root,
 * beside `<UnityCatalogProvider>`. Omitting `service` uses
 * {@link defaultQueryService}.
 */
export function QueryServiceProvider({
  service = defaultQueryService(),
  children,
}: {
  service?: QueryService;
  children: ReactNode;
}) {
  return (
    <QueryServiceContext.Provider value={service}>
      {children}
    </QueryServiceContext.Provider>
  );
}

/**
 * Read the active {@link QueryService}. Falls back to {@link defaultQueryService}
 * when no provider is mounted, so consumers never need a wrapper just to render.
 */
export function useQueryService(): QueryService {
  return useContext(QueryServiceContext) ?? defaultQueryService();
}
