// Unity Catalog client context.
//
// The UC data layer (queries.ts / mutations.ts) reads its client from here via
// `useUnityCatalog()` rather than importing the module singleton, so the host
// decides the base URL / transport / auth by mounting `<UnityCatalogProvider>`.
// This is the dependency inversion described in docs/portable-uc-components.md
// (decision 2) — the seam a future WASM client swaps into with no hook changes.
//
// When no provider is mounted, `useUnityCatalog()` falls back to
// `defaultUnityCatalogClient` (identical config to the historical singleton), so
// tests and Storybook stories need no wrapper and behave exactly as before.
import { createContext, type ReactNode, useContext } from "react";
import { defaultUnityCatalogClient, type UnityCatalogClient } from "../api";

const UnityCatalogContext = createContext<UnityCatalogClient | null>(null);

/**
 * Provide a {@link UnityCatalogClient} to the UC data layer. Mount once near the
 * app root (see main.tsx). Omitting `client` uses {@link defaultUnityCatalogClient}.
 */
export function UnityCatalogProvider({
  client = defaultUnityCatalogClient,
  children,
}: {
  client?: UnityCatalogClient;
  children: ReactNode;
}) {
  return (
    <UnityCatalogContext.Provider value={client}>
      {children}
    </UnityCatalogContext.Provider>
  );
}

/**
 * Read the active {@link UnityCatalogClient}. Falls back to
 * {@link defaultUnityCatalogClient} when no provider is mounted, so consumers
 * never need a wrapper just to render.
 */
export function useUnityCatalog(): UnityCatalogClient {
  return useContext(UnityCatalogContext) ?? defaultUnityCatalogClient;
}
