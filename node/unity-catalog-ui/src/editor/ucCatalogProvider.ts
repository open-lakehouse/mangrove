// The Unity-Catalog-backed `CatalogProvider` for the editor's SQL completion.
//
// This is the concrete implementation of the editor's `CatalogProvider` seam,
// living HERE (in the UC package, which owns the UC client) rather than in the
// editor package — the editor is a leaf and must not depend on Unity Catalog.
// The app registers this via `registerCatalogProvider(ucCatalogProvider)` so
// SQL completion is sourced from the live catalog. See node/app/src/main.tsx.

import type { CatalogProvider } from "@open-lakehouse/editor";
import { fetchClient } from "@open-lakehouse/unity-catalog-client";

const PAGE = 1000;

/** Real provider: Unity Catalog over the shared REST client. */
export const ucCatalogProvider: CatalogProvider = {
  async catalogs() {
    const { data } = await fetchClient.GET("/catalogs", {
      params: { query: { max_results: PAGE } },
    });
    return (data?.catalogs ?? [])
      .map((c) => c.name)
      .filter((n): n is string => !!n);
  },
  async schemas(catalog) {
    const { data } = await fetchClient.GET("/schemas", {
      params: { query: { catalog_name: catalog, max_results: PAGE } },
    });
    return (data?.schemas ?? [])
      .map((s) => s.name)
      .filter((n): n is string => !!n);
  },
  async tables(catalog, schema) {
    const { data } = await fetchClient.GET("/tables", {
      params: {
        query: {
          catalog_name: catalog,
          schema_name: schema,
          max_results: PAGE,
        },
      },
    });
    return (data?.tables ?? [])
      .map((t) => t.name)
      .filter((n): n is string => !!n);
  },
  async columns(fullTableName) {
    const { data } = await fetchClient.GET("/tables/{full_name}", {
      params: { path: { full_name: fullTableName } },
    });
    return (data?.columns ?? [])
      .filter((c) => !!c.name)
      .map((c) => ({ name: c.name as string, type: c.type_text ?? "" }));
  },
};
