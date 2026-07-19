// Pluggable catalog-metadata seam — the single swap point that lets a host feed
// live catalog/schema/table/column names to SQL completion WITHOUT the editor
// depending on where those names come from (a Unity Catalog REST client, a
// fixture, a downstream service).
//
// This mirrors the query-runner seam in `@open-lakehouse/query`: a module
// singleton `current`, a `registerCatalogProvider` swap point, a `has*` gate,
// and a late-binding `getCatalogProvider` the completion layer always calls.
//
// The default is an EMPTY provider (returns `[]`), NOT a throwing one: SQL
// completion must degrade to keyword-only when no catalog is wired, never break
// typing. `hasCatalogProvider()` and `NoCatalogProviderError` are exported for
// hosts that want to assert a provider is present.

export interface CatalogColumn {
  name: string;
  type: string;
}

/** The metadata the SQL completion service needs. Host-provided. */
export interface CatalogProvider {
  catalogs(): Promise<string[]>;
  schemas(catalog: string): Promise<string[]>;
  tables(catalog: string, schema: string): Promise<string[]>;
  /** Columns for a fully-qualified `catalog.schema.table`. */
  columns(fullTableName: string): Promise<CatalogColumn[]>;
}

/** Thrown only by hosts that opt in to asserting a provider is registered; the
 *  editor itself never throws it (completion degrades to keyword-only instead). */
export class NoCatalogProviderError extends Error {
  constructor() {
    super(
      "No catalog provider registered. SQL name completion needs one installed " +
        "via registerCatalogProvider (the Unity Catalog package does this). " +
        "Until then, only keyword completion is offered.",
    );
    this.name = "NoCatalogProviderError";
  }
}

/** Default: an empty catalog. Returns `[]` rather than throwing so completion
 *  silently degrades to keywords-only when no host provider is registered. */
const emptyCatalogProvider: CatalogProvider = {
  catalogs: async () => [],
  schemas: async () => [],
  tables: async () => [],
  columns: async () => [],
};

let current: CatalogProvider = emptyCatalogProvider;

/** Install the catalog provider. Hosts call this once, before the editor
 *  bootstraps (late binding below tolerates any ordering). */
export function registerCatalogProvider(provider: CatalogProvider): void {
  current = provider;
}

/** The provider currently in effect (the registered one, or the empty default). */
export function getCatalogProvider(): CatalogProvider {
  return current;
}

/** True once a real provider has been registered — the gate a host can read to
 *  decide whether catalog-aware completion is available. */
export function hasCatalogProvider(): boolean {
  return current !== emptyCatalogProvider;
}
