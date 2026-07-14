import {
  Button,
  cn,
  Input,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import {
  objectFullName,
  useFunctions,
  useModels,
  useSchemaDetail,
  useTables,
  useVolumes,
} from "@open-lakehouse/unity-catalog-client";
import type { UseInfiniteQueryResult } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useEffect, useState } from "react";

import { GROUPS, type GroupDef } from "../groups";
import { useCatalogSelection } from "../selection";
import type { ObjectKind } from "../types";
import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid, toEpochMillis } from "./Meta";

// Minimal shape shared by every child list item we render in the table. The
// per-kind list payloads are wider than this, but Name / Owner / Created are the
// common columns, and full_name (+ namespace parts) address the row on click.
interface ObjectRow {
  name?: string;
  owner?: string;
  created_at?: number;
  full_name?: string;
  catalog_name?: string;
  schema_name?: string;
}

type ChildQuery = UseInfiniteQueryResult<ObjectRow[], unknown>;

// Subscribe to all four child lists ONCE, keyed identically to the tree's own
// list hooks — so expanding the schema in the tree and opening it here share a
// single cache entry (and a single request) per kind. Results flow down as
// props; nothing below re-subscribes. Hooks run unconditionally and in a stable
// order (rules of hooks), before any early return in the caller.
function useSchemaChildren(
  catalog: string | undefined,
  schema: string | undefined,
): Record<ObjectKind, ChildQuery> {
  const table = useTables(catalog, schema);
  const volume = useVolumes(catalog, schema);
  const fn = useFunctions(catalog, schema);
  const model = useModels(catalog, schema);
  return {
    table,
    volume,
    function: fn,
    model,
  } as unknown as Record<ObjectKind, ChildQuery>;
}

export function SchemaDetail({ fullName }: { fullName: string }) {
  const { data: schema, isLoading, error } = useSchemaDetail(fullName);
  // Active kind lives in the URL (`tab`), so the tree's kind rows, deep links,
  // and this filter bar all drive the same state. Defaults to tables.
  const { schemaTab, setSchemaTab } = useCatalogSelection();
  const activeKind: ObjectKind = schemaTab ?? "table";

  // A schema's fullName is always `catalog.schema`; derive the list params from
  // it directly so children start loading in parallel with the schema detail
  // (no dependency on the detail response resolving first).
  const [catalogName, schemaName] = fullName.split(".");
  const children = useSchemaChildren(catalogName, schemaName);

  // Which top-level view is showing. Selecting a child kind from the tree (or a
  // deep link) sets `schemaTab`, which should always surface the Overview list —
  // so we snap back to it whenever that changes, while still letting the user
  // toggle to Details manually.
  const [view, setView] = useState<"overview" | "details">("overview");
  useEffect(() => {
    if (schemaTab) setView("overview");
  }, [schemaTab]);

  if (!schema) return <DetailStates isLoading={isLoading} error={error} />;

  // The server only returns a schema's storage fields when it was created with
  // an explicit location; an inherited schema reports neither and resolves its
  // managed storage from the parent catalog at write time.
  const inheritsStorage = !schema.storage_root && !schema.storage_location;
  const activeGroup = GROUPS.find((g) => g.kind === activeKind) ?? GROUPS[0];

  return (
    <Tabs
      value={view}
      onValueChange={(v) => setView(v as "overview" | "details")}
    >
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="details">Details</TabsTrigger>
      </TabsList>

      <TabsContent value="overview" className="space-y-3">
        {/* Filter bar: pick which child securable kind to list below. Counts
            read straight from the shared query results (no extra fetches). */}
        <div className="flex flex-wrap items-center gap-1 border-b pb-2">
          {GROUPS.map((group) => (
            <KindTab
              key={group.kind}
              group={group}
              query={children[group.kind]}
              active={group.kind === activeKind}
              onSelect={() => setSchemaTab(group.kind)}
            />
          ))}
        </div>

        {/* Remount per kind so the name filter resets when switching tabs. */}
        <KindObjects
          key={activeGroup.kind}
          group={activeGroup}
          query={children[activeKind]}
        />
      </TabsContent>

      <TabsContent value="details">
        <MetaGrid>
          <Meta label="Owner" value={schema.owner} />
          <Meta label="Catalog" value={schema.catalog_name} />
          <Meta label="Storage root" value={schema.storage_root} wide mono />
          {inheritsStorage ? (
            <Meta
              label="Storage location"
              value="Inherited from parent catalog"
              wide
            />
          ) : (
            <Meta
              label="Storage location"
              value={schema.storage_location}
              wide
              mono
            />
          )}
          <Meta label="Comment" value={schema.comment} wide />
        </MetaGrid>
      </TabsContent>
    </Tabs>
  );
}

function KindTab({
  group,
  query,
  active,
  onSelect,
}: {
  group: GroupDef;
  query: ChildQuery;
  active: boolean;
  onSelect: () => void;
}) {
  const count = query.data?.length ?? 0;
  const Icon = group.Icon;

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "flex items-center gap-1.5 rounded-md px-2.5 py-1 text-sm transition-colors",
        active
          ? "bg-muted font-medium text-foreground"
          : "text-muted-foreground hover:bg-muted/60",
      )}
    >
      <Icon className="h-4 w-4" />
      {group.title}
      <span className="font-mono text-xs tabular-nums text-muted-foreground">
        {count}
        {query.hasNextPage ? "+" : ""}
      </span>
    </button>
  );
}

function KindObjects({ group, query }: { group: GroupDef; query: ChildQuery }) {
  const { select } = useCatalogSelection();
  const {
    data,
    isLoading,
    error,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = query;
  const [filter, setFilter] = useState("");

  const rows = data ?? [];
  const needle = filter.trim().toLowerCase();
  const filtered = needle
    ? rows.filter((r) => r.name?.toLowerCase().includes(needle))
    : rows;
  const label = group.title.toLowerCase();
  const Icon = group.Icon;

  return (
    <div className="space-y-2">
      <div className="relative max-w-xs">
        <Search className="pointer-events-none absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder={`Filter ${label}`}
          className="h-8 pl-7 text-sm"
        />
      </div>

      {error ? (
        <p className="text-sm text-destructive">Failed to load {label}.</p>
      ) : isLoading ? (
        <p className="text-sm text-muted-foreground">Loading…</p>
      ) : filtered.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {rows.length === 0
            ? `No ${label} in this schema.`
            : `No ${label} match "${filter.trim()}".`}
        </p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left text-xs text-muted-foreground">
              <th className="py-1.5 pr-4 font-medium">Name</th>
              <th className="py-1.5 pr-4 font-medium">Owner</th>
              <th className="py-1.5 font-medium">Created</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((row) => {
              const full = row.full_name || objectFullName(row);
              return (
                <tr
                  key={full}
                  onClick={() => select({ kind: group.kind, fullName: full })}
                  className="cursor-pointer border-b last:border-b-0 hover:bg-accent"
                >
                  <td className="py-1.5 pr-4">
                    <span className="flex items-center gap-2">
                      <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
                      <span className="truncate font-medium">{row.name}</span>
                    </span>
                  </td>
                  <td className="py-1.5 pr-4 text-muted-foreground">
                    {row.owner || "—"}
                  </td>
                  <td className="py-1.5 text-muted-foreground">
                    {formatCreated(row.created_at)}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {hasNextPage && (
        <Button
          variant="ghost"
          size="sm"
          className="text-xs"
          disabled={isFetchingNextPage}
          onClick={() => fetchNextPage()}
        >
          {isFetchingNextPage ? "Loading…" : "Load more"}
        </Button>
      )}
    </div>
  );
}

function formatCreated(ms?: number | string | null): string {
  const epoch = toEpochMillis(ms);
  if (epoch === undefined) return "—";
  return new Date(epoch).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
  });
}
