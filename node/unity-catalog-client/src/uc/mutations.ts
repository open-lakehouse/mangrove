// Unity Catalog invalidation map.
//
// The three-level namespace is a hierarchy: catalog -> schema -> table. When a
// resource changes, the lists that contain it must be refetched. Mutations
// (create/update/delete) are not implemented yet for the read-first navigation
// surface, but the invalidation strategy is defined here so wiring them up later
// is mechanical: a mutation's `onSettled` just calls the matching helper.
//
// We match by PREDICATE on the canonical ["get", path, init] key rather than by
// exact key, so a single call invalidates every page / param variant of a list
// (e.g. all `maxResults`/`pageToken` combinations).

import type { QueryClient, QueryKey } from "@tanstack/react-query";
import { useQueryClient } from "@tanstack/react-query";
import { useUnityCatalog } from "./context";
import {
  catalogDetailQuery,
  credentialDetailQuery,
  externalLocationDetailQuery,
  modelDetailQuery,
  tableDetailQuery,
  volumeDetailQuery,
} from "./queries";

interface ListInit {
  params?: { query?: { catalog_name?: string; schema_name?: string } };
}

function listQuery(key: QueryKey, path: string): ListInit | undefined {
  if (!Array.isArray(key) || key[0] !== "get" || key[1] !== path) {
    return undefined;
  }
  return key[2] as ListInit | undefined;
}

/** Invalidate the top-level catalog list. */
export function invalidateCatalogs(queryClient: QueryClient) {
  return queryClient.invalidateQueries({
    predicate: (q) => !!listQuery(q.queryKey, "/catalogs"),
  });
}

/** Invalidate the metastore credential list. */
export function invalidateCredentials(queryClient: QueryClient) {
  return queryClient.invalidateQueries({
    predicate: (q) => !!listQuery(q.queryKey, "/credentials"),
  });
}

/** Invalidate the metastore external-location list. */
export function invalidateExternalLocations(queryClient: QueryClient) {
  return queryClient.invalidateQueries({
    predicate: (q) => !!listQuery(q.queryKey, "/external-locations"),
  });
}

/** Invalidate every schema list for a catalog (all pages/params). */
export function invalidateSchemas(
  queryClient: QueryClient,
  catalogName: string,
) {
  return queryClient.invalidateQueries({
    predicate: (q) =>
      listQuery(q.queryKey, "/schemas")?.params?.query?.catalog_name ===
      catalogName,
  });
}

/** Invalidate a schema-scoped list (tables/volumes/functions/models). */
function invalidateSchemaList(
  queryClient: QueryClient,
  path: string,
  catalogName: string,
  schemaName: string,
) {
  return queryClient.invalidateQueries({
    predicate: (q) => {
      const query = listQuery(q.queryKey, path)?.params?.query;
      return (
        query?.catalog_name === catalogName && query?.schema_name === schemaName
      );
    },
  });
}

/** Invalidate every table list for a schema (all pages/params). */
export function invalidateTables(
  queryClient: QueryClient,
  catalogName: string,
  schemaName: string,
) {
  return invalidateSchemaList(queryClient, "/tables", catalogName, schemaName);
}

/** Invalidate every volume list for a schema. */
export function invalidateVolumes(
  queryClient: QueryClient,
  catalogName: string,
  schemaName: string,
) {
  return invalidateSchemaList(queryClient, "/volumes", catalogName, schemaName);
}

/** Invalidate every function list for a schema. */
export function invalidateFunctions(
  queryClient: QueryClient,
  catalogName: string,
  schemaName: string,
) {
  return invalidateSchemaList(
    queryClient,
    "/functions",
    catalogName,
    schemaName,
  );
}

/** Invalidate every registered-model list for a schema. */
export function invalidateModels(
  queryClient: QueryClient,
  catalogName: string,
  schemaName: string,
) {
  return invalidateSchemaList(queryClient, "/models", catalogName, schemaName);
}

/** Drop a single catalog's detail cache. */
export function removeCatalogDetail(queryClient: QueryClient, name: string) {
  queryClient.removeQueries({ queryKey: catalogDetailQuery(name).queryKey });
}

/** Drop a single table's detail cache. */
export function removeTableDetail(queryClient: QueryClient, fullName: string) {
  queryClient.removeQueries({ queryKey: tableDetailQuery(fullName).queryKey });
}

/**
 * Remove all cached descendants of a catalog (its schema lists and any table
 * lists under it). Use after deleting a catalog so stale child data can't be
 * served from cache.
 */
export function removeCatalogDescendants(
  queryClient: QueryClient,
  catalogName: string,
) {
  queryClient.removeQueries({
    predicate: (q) => {
      for (const path of [
        "/schemas",
        "/tables",
        "/volumes",
        "/functions",
        "/models",
      ]) {
        if (
          listQuery(q.queryKey, path)?.params?.query?.catalog_name ===
          catalogName
        ) {
          return true;
        }
      }
      return false;
    },
  });
}

// ── Create mutations ────────────────────────────────────────────────────────
//
// Each hook wraps `$api.useMutation("post", path)` and, on success, invalidates
// the parent list so the tree refetches in place. Mutations never hand-write
// keys; they reuse the same predicate-based invalidators as everything else.

/** Create a catalog, then refresh the catalog list. */
export function useCreateCatalog() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/catalogs", {
    onSuccess: () => invalidateCatalogs(queryClient),
  });
}

/** Create a schema, then refresh its parent catalog's schema list. */
export function useCreateSchema() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/schemas", {
    onSuccess: (data) => {
      if (data?.catalog_name) invalidateSchemas(queryClient, data.catalog_name);
    },
  });
}

/** Create a volume, then refresh its parent schema's volume list. */
export function useCreateVolume() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/volumes", {
    onSuccess: (data) => {
      if (data?.catalog_name && data?.schema_name) {
        invalidateVolumes(queryClient, data.catalog_name, data.schema_name);
      }
    },
  });
}

/** Create a registered model, then refresh its parent schema's model list. */
export function useCreateRegisteredModel() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/models", {
    onSuccess: (data) => {
      if (data?.catalog_name && data?.schema_name) {
        invalidateModels(queryClient, data.catalog_name, data.schema_name);
      }
    },
  });
}

// ── Optimistic list removal (for deletes) ───────────────────────────────────
//
// Deletes feel instant: we drop the row from every cached page of the matching
// list immediately, snapshot what we removed, and restore it verbatim if the
// request fails. `onSettled` then invalidates to reconcile with the server.

interface InfiniteListSnapshot {
  key: QueryKey;
  data: unknown;
}

function optimisticRemove(
  queryClient: QueryClient,
  path: string,
  listField: string,
  matches: (item: { name?: string; full_name?: string }) => boolean,
): InfiniteListSnapshot[] {
  const entries = queryClient.getQueriesData({
    predicate: (q) => !!listQuery(q.queryKey, path),
  });
  const snapshots: InfiniteListSnapshot[] = [];

  for (const [key, data] of entries) {
    if (!data) continue;
    snapshots.push({ key, data });
    const inf = data as { pages?: Record<string, unknown>[] };
    if (!inf.pages) continue;
    queryClient.setQueryData(key, {
      ...inf,
      pages: inf.pages.map((page) => ({
        ...page,
        [listField]: ((page[listField] as { name?: string }[]) ?? []).filter(
          (item) => !matches(item),
        ),
      })),
    });
  }

  return snapshots;
}

function restore(queryClient: QueryClient, snapshots: InfiniteListSnapshot[]) {
  for (const { key, data } of snapshots) {
    queryClient.setQueryData(key, data);
  }
}

// ── Update (PATCH) mutations ─────────────────────────────────────────────────
//
// On success we write the returned info into its detail cache and invalidate the
// parent list (a rename changes the row identity, so the list must refetch).

/** Update a catalog's comment / name. */
export function useUpdateCatalog() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/catalogs/{name}", {
    onSuccess: (data) => {
      if (data?.name) {
        queryClient.setQueryData(catalogDetailQuery(data.name).queryKey, data);
      }
      invalidateCatalogs(queryClient);
    },
  });
}

/** Update a schema's comment / name. */
export function useUpdateSchema() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/schemas/{full_name}", {
    onSuccess: (data) => {
      if (data?.catalog_name) invalidateSchemas(queryClient, data.catalog_name);
    },
  });
}

/** Update a volume's comment / name. */
export function useUpdateVolume() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/volumes/{name}", {
    onSuccess: (data) => {
      if (data?.full_name) {
        queryClient.setQueryData(
          volumeDetailQuery(data.full_name).queryKey,
          data,
        );
      }
      if (data?.catalog_name && data?.schema_name) {
        invalidateVolumes(queryClient, data.catalog_name, data.schema_name);
      }
    },
  });
}

/** Update a registered model's comment / name. */
export function useUpdateRegisteredModel() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/models/{full_name}", {
    onSuccess: (data) => {
      if (data?.full_name) {
        queryClient.setQueryData(
          modelDetailQuery(data.full_name).queryKey,
          data,
        );
      }
      if (data?.catalog_name && data?.schema_name) {
        invalidateModels(queryClient, data.catalog_name, data.schema_name);
      }
    },
  });
}

// ── Delete mutations (optimistic) ────────────────────────────────────────────

/** Delete a catalog (and purge its cached descendants). */
export function useDeleteCatalog() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("delete", "/catalogs/{name}", {
    onMutate: ({ params }) => {
      const name = params.path.name;
      const snapshots = optimisticRemove(
        queryClient,
        "/catalogs",
        "catalogs",
        (item) => item.name === name,
      );
      return { snapshots };
    },
    onError: (_err, _vars, context) => {
      if (context?.snapshots) restore(queryClient, context.snapshots);
    },
    onSuccess: (_data, { params }) => {
      removeCatalogDetail(queryClient, params.path.name);
      removeCatalogDescendants(queryClient, params.path.name);
    },
    onSettled: () => invalidateCatalogs(queryClient),
  });
}

/** Delete a schema. */
export function useDeleteSchema() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("delete", "/schemas/{full_name}", {
    onMutate: ({ params }) => {
      const fullName = params.path.full_name;
      const name = fullName.split(".").pop();
      const snapshots = optimisticRemove(
        queryClient,
        "/schemas",
        "schemas",
        (item) => item.name === name || item.full_name === fullName,
      );
      return { snapshots, catalog: fullName.split(".")[0] };
    },
    onError: (_err, _vars, context) => {
      if (context?.snapshots) restore(queryClient, context.snapshots);
    },
    onSettled: (_data, _err, { params }) => {
      invalidateSchemas(queryClient, params.path.full_name.split(".")[0]);
    },
  });
}

// Shared optimistic-delete wiring for a schema-scoped leaf list. The caller
// supplies the canonical list path + the field that holds the rows, plus the
// fully-qualified name being deleted.
function leafDeleteHandlers(
  queryClient: QueryClient,
  listPath: string,
  listField: string,
) {
  return {
    onMutate: (fullName: string) => {
      const name = fullName.split(".").pop();
      const snapshots = optimisticRemove(
        queryClient,
        listPath,
        listField,
        (item) => item.full_name === fullName || item.name === name,
      );
      return { snapshots };
    },
    onError: (context: { snapshots?: InfiniteListSnapshot[] } | undefined) => {
      if (context?.snapshots) restore(queryClient, context.snapshots);
    },
    onSettled: (fullName: string) => {
      const [catalog, schema] = fullName.split(".");
      if (catalog && schema) {
        invalidateSchemaList(queryClient, listPath, catalog, schema);
      }
    },
  };
}

/** Delete a table. */
export function useDeleteTable() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  const h = leafDeleteHandlers(queryClient, "/tables", "tables");
  return $api.useMutation("delete", "/tables/{full_name}", {
    onMutate: ({ params }) => h.onMutate(params.path.full_name),
    onError: (_e, _v, ctx) => h.onError(ctx),
    onSettled: (_d, _e, { params }) => h.onSettled(params.path.full_name),
  });
}

/** Delete a volume. */
export function useDeleteVolume() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  const h = leafDeleteHandlers(queryClient, "/volumes", "volumes");
  return $api.useMutation("delete", "/volumes/{name}", {
    onMutate: ({ params }) => h.onMutate(params.path.name),
    onError: (_e, _v, ctx) => h.onError(ctx),
    onSettled: (_d, _e, { params }) => h.onSettled(params.path.name),
  });
}

/** Delete a function. */
export function useDeleteFunction() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  const h = leafDeleteHandlers(queryClient, "/functions", "functions");
  return $api.useMutation("delete", "/functions/{name}", {
    onMutate: ({ params }) => h.onMutate(params.path.name),
    onError: (_e, _v, ctx) => h.onError(ctx),
    onSettled: (_d, _e, { params }) => h.onSettled(params.path.name),
  });
}

/** Delete a registered model. */
export function useDeleteRegisteredModel() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  const h = leafDeleteHandlers(queryClient, "/models", "registered_models");
  return $api.useMutation("delete", "/models/{full_name}", {
    onMutate: ({ params }) => h.onMutate(params.path.full_name),
    onError: (_e, _v, ctx) => h.onError(ctx),
    onSettled: (_d, _e, { params }) => h.onSettled(params.path.full_name),
  });
}

// ── Storage credentials & external locations (metastore-level) ───────────────

/** Create a storage credential, then refresh the credential list. */
export function useCreateCredential() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/credentials", {
    onSuccess: () => invalidateCredentials(queryClient),
  });
}

/** Update a credential's comment / IAM role / name. */
export function useUpdateCredential() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/credentials/{name}", {
    onSuccess: (data) => {
      if (data?.name) {
        queryClient.setQueryData(
          credentialDetailQuery(data.name).queryKey,
          data,
        );
      }
      invalidateCredentials(queryClient);
    },
  });
}

/** Delete a credential. */
export function useDeleteCredential() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("delete", "/credentials/{name}", {
    onMutate: ({ params }) => {
      const name = params.path.name;
      const snapshots = optimisticRemove(
        queryClient,
        "/credentials",
        "credentials",
        (item) => item.name === name,
      );
      return { snapshots };
    },
    onError: (_err, _vars, context) => {
      if (context?.snapshots) restore(queryClient, context.snapshots);
    },
    onSuccess: (_data, { params }) => {
      queryClient.removeQueries({
        queryKey: credentialDetailQuery(params.path.name).queryKey,
      });
    },
    onSettled: () => invalidateCredentials(queryClient),
  });
}

/** Create an external location, then refresh the external-location list. */
export function useCreateExternalLocation() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("post", "/external-locations", {
    onSuccess: () => invalidateExternalLocations(queryClient),
  });
}

/** Update an external location's url / credential / comment / name. */
export function useUpdateExternalLocation() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("patch", "/external-locations/{name}", {
    onSuccess: (data) => {
      if (data?.name) {
        queryClient.setQueryData(
          externalLocationDetailQuery(data.name).queryKey,
          data,
        );
      }
      invalidateExternalLocations(queryClient);
    },
  });
}

/** Delete an external location. */
export function useDeleteExternalLocation() {
  const { $api } = useUnityCatalog();
  const queryClient = useQueryClient();
  return $api.useMutation("delete", "/external-locations/{name}", {
    onMutate: ({ params }) => {
      const name = params.path.name;
      const snapshots = optimisticRemove(
        queryClient,
        "/external-locations",
        "external_locations",
        (item) => item.name === name,
      );
      return { snapshots };
    },
    onError: (_err, _vars, context) => {
      if (context?.snapshots) restore(queryClient, context.snapshots);
    },
    onSuccess: (_data, { params }) => {
      queryClient.removeQueries({
        queryKey: externalLocationDetailQuery(params.path.name).queryKey,
      });
    },
    onSettled: () => invalidateExternalLocations(queryClient),
  });
}
