// Pure cache helpers behind the mutation hooks in ./mutations.ts: matching
// list queries by path and parent scope, optimistic list removal with
// rollback, and detail lookups by name prefix. Kept out of the public
// `./react` entry point; exported here so they can be tested directly.

import type { QueryClient, QueryKey } from "@tanstack/react-query";

export interface ListInit {
  params?: { query?: { catalog_name?: string; schema_name?: string } };
}

export function listQuery(key: QueryKey, path: string): ListInit | undefined {
  if (!Array.isArray(key) || key[0] !== "get" || key[1] !== path) {
    return undefined;
  }
  return key[2] as ListInit | undefined;
}

/** Parent scope of a list, as carried in its query params. */
export interface ListScope {
  catalog_name?: string;
  schema_name?: string;
}

/** True if a list's query params place it inside `scope`. */
export function inScope(init: ListInit, scope: ListScope): boolean {
  const query = init.params?.query;
  if (scope.catalog_name !== undefined) {
    if (query?.catalog_name !== scope.catalog_name) return false;
  }
  if (scope.schema_name !== undefined) {
    if (query?.schema_name !== scope.schema_name) return false;
  }
  return true;
}

/** The parent scope of a schema-scoped full name (`catalog.schema.leaf`). */
export function leafScope(fullName: string): ListScope {
  const [catalog_name, schema_name] = fullName.split(".");
  return { catalog_name, schema_name };
}

/** True for a detail query whose path id lies under `prefix` (`a.` / `a.b.`). */
export function isDetailUnder(key: QueryKey, prefix: string): boolean {
  if (!Array.isArray(key) || key[0] !== "get") return false;
  const path = (key[2] as { params?: { path?: Record<string, unknown> } })
    ?.params?.path;
  if (!path) return false;
  return Object.values(path).some(
    (value) => typeof value === "string" && value.startsWith(prefix),
  );
}

export interface InfiniteListSnapshot {
  key: QueryKey;
  data: unknown;
}

// Only lists inside `scope` are touched: deleting `a.s.t` must not drop `b.s.t`
// from another schema's cached list just because the leaf names match.
export function optimisticRemove(
  queryClient: QueryClient,
  path: string,
  listField: string,
  scope: ListScope,
  matches: (item: { name?: string; full_name?: string }) => boolean,
): InfiniteListSnapshot[] {
  const entries = queryClient.getQueriesData({
    predicate: (q) => {
      const init = listQuery(q.queryKey, path);
      return !!init && inScope(init, scope);
    },
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

export function restore(
  queryClient: QueryClient,
  snapshots: InfiniteListSnapshot[],
) {
  for (const { key, data } of snapshots) {
    queryClient.setQueryData(key, data);
  }
}
