// Log-query-service context — the injection point for a `LogQueryService`,
// mirroring @open-lakehouse/query's `QueryServiceProvider` / `useQueryService`.
//
// The UI reads its service from here via `useLogQueryService()` rather than
// importing a singleton, so a host decides which service (and thus which runner)
// is in effect by mounting `<LogQueryServiceProvider>`. When no provider is
// mounted, `useLogQueryService()` falls back to `defaultLogQueryService()`, so
// tests and Storybook stories need no wrapper.

import { createContext, type ReactNode, useContext } from "react";
import { defaultLogQueryService } from "./api";
import type { LogQueryService } from "./types";

const LogQueryServiceContext = createContext<LogQueryService | null>(null);

/**
 * Provide a {@link LogQueryService} to the Delta-log UI. Mount once near the app
 * root, beside `<QueryServiceProvider>`. Omitting `service` uses
 * {@link defaultLogQueryService}.
 */
export function LogQueryServiceProvider({
  service = defaultLogQueryService(),
  children,
}: {
  service?: LogQueryService;
  children: ReactNode;
}) {
  return (
    <LogQueryServiceContext.Provider value={service}>
      {children}
    </LogQueryServiceContext.Provider>
  );
}

/**
 * Read the active {@link LogQueryService}. Falls back to
 * {@link defaultLogQueryService} when no provider is mounted, so consumers never
 * need a wrapper just to render.
 */
export function useLogQueryService(): LogQueryService {
  return useContext(LogQueryServiceContext) ?? defaultLogQueryService();
}
