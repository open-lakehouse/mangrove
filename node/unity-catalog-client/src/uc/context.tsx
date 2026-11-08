// Unity Catalog client context.
//
// The UC data layer (queries.ts / mutations.ts) reads its client from here via
// `useUnityCatalog()` rather than importing the module singleton, so the host
// decides the base URL / transport / auth by mounting `<UnityCatalogProvider>`.
// This is also the seam a future WASM client swaps into with no hook changes.
//
// Without an explicit client, the provider takes the transport from the
// `@open-lakehouse/host` contract: the `unity-catalog` binding of the host above
// it (`<HostProvider>`) or of the registered one (`registerHost`). With no host
// either, it uses `defaultUnityCatalogClient` over the platform `fetch`.
//
// When no provider is mounted, `useUnityCatalog()` falls back to
// `defaultUnityCatalogClient`, which follows the registered host too, so tests
// and stories need no wrapper.
import { useOptionalHost } from "@open-lakehouse/host/react";
import { createContext, type ReactNode, useContext, useMemo } from "react";
import {
  createUnityCatalogClient,
  defaultUnityCatalogClient,
  type UnityCatalogClient,
} from "../api.js";
import { unityCatalogFetchFor } from "../fetch.js";

const UnityCatalogContext = createContext<UnityCatalogClient | null>(null);

/**
 * Provide a {@link UnityCatalogClient} to the UC data layer. Mount once near the
 * app root. The client is, in order:
 *
 * 1. `client`, when given;
 * 2. else a client over the `unity-catalog` binding of the nearest host
 *    (`<HostProvider>`, else `registerHost`), looked up on every request, so an
 *    environment switch needs no new client;
 * 3. else {@link defaultUnityCatalogClient}.
 */
export function UnityCatalogProvider({
  client,
  children,
}: {
  client?: UnityCatalogClient;
  children: ReactNode;
}) {
  const host = useOptionalHost();
  const value = useMemo(() => {
    if (client) return client;
    if (!host) return defaultUnityCatalogClient;
    return createUnityCatalogClient({
      fetch: unityCatalogFetchFor(() => host),
    });
  }, [client, host]);
  return (
    <UnityCatalogContext.Provider value={value}>
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
