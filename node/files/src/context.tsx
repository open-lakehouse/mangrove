// Files-service context — the injection point for a `FilesService`, mirroring
// @open-lakehouse/query's `QueryServiceProvider` / `useQueryService`.
//
// The UI reads its service from here via `useFilesService()` rather than
// importing a singleton, so a host decides which service (and thus which runner)
// is in effect by mounting `<FilesServiceProvider>`. When no provider is mounted,
// `useFilesService()` falls back to `defaultFilesService()`, so tests and
// Storybook stories need no wrapper.

import { createContext, type ReactNode, useContext } from "react";
import { defaultFilesService } from "./api";
import type { FilesService } from "./types";

const FilesServiceContext = createContext<FilesService | null>(null);

/**
 * Provide a {@link FilesService} to the volume-files UI. Mount once near the app
 * root, beside `<UnityCatalogProvider>`. Omitting `service` uses
 * {@link defaultFilesService}.
 */
export function FilesServiceProvider({
  service = defaultFilesService(),
  children,
}: {
  service?: FilesService;
  children: ReactNode;
}) {
  return (
    <FilesServiceContext.Provider value={service}>
      {children}
    </FilesServiceContext.Provider>
  );
}

/**
 * Read the active {@link FilesService}. Falls back to {@link defaultFilesService}
 * when no provider is mounted, so consumers never need a wrapper just to render.
 */
export function useFilesService(): FilesService {
  return useContext(FilesServiceContext) ?? defaultFilesService();
}
