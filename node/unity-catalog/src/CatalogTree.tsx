import {
  Button,
  cn,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@open-lakehouse/ui-kit";
import {
  objectFullName,
  prefetchSchemas,
  useCatalogs,
  useSchemas,
} from "@open-lakehouse/unity-catalog-client";
import { useQueryClient } from "@tanstack/react-query";
import {
  Database,
  FolderTree,
  Globe,
  KeyRound,
  Pencil,
  Plus,
  Settings,
  Trash2,
} from "lucide-react";
import { useCatalogDialogs } from "./dialogs";
import { nodeId, useExpansion } from "./ExpansionContext";
import { GROUPS, type GroupDef } from "./groups";
import { PANE_HEADER_CLASS } from "./layout";
import { RowMenu } from "./RowMenu";
import { SectionLabel } from "./SectionLabel";
import { useCatalogSelection } from "./selection";
import { CreateAction, ListStates, TreeRow } from "./TreeRow";

export function CatalogTree() {
  const queryClient = useQueryClient();
  const catalogs = useCatalogs();
  const dialogs = useCatalogDialogs();

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className={cn(PANE_HEADER_CLASS, "px-3")}>
        <SectionLabel>Catalogs</SectionLabel>
        <div className="flex items-center gap-1">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0"
                aria-label="Metastore settings"
                title="Metastore settings"
              >
                <Settings className="h-3.5 w-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuLabel>Add storage securable</DropdownMenuLabel>
              <DropdownMenuItem
                onSelect={() => dialogs.create({ kind: "external_location" })}
              >
                <Globe />
                External location
              </DropdownMenuItem>
              <DropdownMenuItem
                onSelect={() => dialogs.create({ kind: "credential" })}
              >
                <KeyRound />
                Credential
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            aria-label="New catalog"
            title="New catalog"
            onClick={() => dialogs.create({ kind: "catalog" })}
          >
            <Plus className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-1">
        <ListStates
          isLoading={catalogs.isLoading}
          error={catalogs.error}
          isEmpty={(catalogs.data?.length ?? 0) === 0}
          hasNextPage={catalogs.hasNextPage}
          isFetchingNextPage={catalogs.isFetchingNextPage}
          onLoadMore={() => catalogs.fetchNextPage()}
        >
          {catalogs.data?.map((catalog) => (
            <CatalogNode
              key={catalog.name}
              name={catalog.name ?? ""}
              comment={catalog.comment}
              onPrefetch={() =>
                catalog.name && prefetchSchemas(queryClient, catalog.name)
              }
            />
          ))}
        </ListStates>
      </div>
    </div>
  );
}

function CatalogNode({
  name,
  comment,
  onPrefetch,
}: {
  name: string;
  comment?: string;
  onPrefetch: () => void;
}) {
  const { isOpen, toggle } = useExpansion();
  const { selection, select } = useCatalogSelection();
  const dialogs = useCatalogDialogs();
  const id = nodeId.catalog(name);
  const selected = selection?.kind === "catalog" && selection.fullName === name;

  return (
    <div>
      <TreeRow
        depth={0}
        icon={<Database className="h-4 w-4 text-muted-foreground" />}
        label={name}
        expandable
        open={isOpen(id)}
        selected={selected}
        onToggle={() => toggle(id)}
        onSelect={() => select({ kind: "catalog", fullName: name })}
        onMouseEnter={onPrefetch}
        action={
          <>
            <CreateAction
              title="New schema"
              onClick={() => dialogs.create({ kind: "schema", catalog: name })}
            />
            <RowMenu
              label={`Catalog ${name} actions`}
              items={[
                {
                  label: "Edit",
                  icon: <Pencil />,
                  onSelect: () =>
                    dialogs.edit({ kind: "catalog", name, comment }),
                },
                {
                  label: "Delete",
                  icon: <Trash2 />,
                  variant: "destructive",
                  separatorBefore: true,
                  onSelect: () => dialogs.remove({ kind: "catalog", name }),
                },
              ]}
            />
          </>
        }
      />
      {isOpen(id) && <SchemaList catalog={name} />}
    </div>
  );
}

function SchemaList({ catalog }: { catalog: string }) {
  const schemas = useSchemas(catalog);
  return (
    <ListStates
      depth={1}
      isLoading={schemas.isLoading}
      error={schemas.error}
      isEmpty={(schemas.data?.length ?? 0) === 0}
      hasNextPage={schemas.hasNextPage}
      isFetchingNextPage={schemas.isFetchingNextPage}
      onLoadMore={() => schemas.fetchNextPage()}
    >
      {schemas.data?.map((schema) => (
        <SchemaNode
          key={schema.name}
          catalog={catalog}
          schema={schema.name ?? ""}
          comment={schema.comment}
        />
      ))}
    </ListStates>
  );
}

function SchemaNode({
  catalog,
  schema,
  comment,
}: {
  catalog: string;
  schema: string;
  comment?: string;
}) {
  const { isOpen, toggle } = useExpansion();
  const { selection, schemaTab, select } = useCatalogSelection();
  const dialogs = useCatalogDialogs();
  const id = nodeId.schema(catalog, schema);
  const fullName = `${catalog}.${schema}`;
  // The schema row itself is "selected" only when a schema is shown without a
  // specific kind tab; selecting a kind row highlights that row instead.
  const selected =
    selection?.kind === "schema" &&
    selection.fullName === fullName &&
    !schemaTab;

  return (
    <div>
      <TreeRow
        depth={1}
        icon={<FolderTree className="h-4 w-4 text-muted-foreground" />}
        label={schema}
        expandable
        open={isOpen(id)}
        selected={selected}
        onToggle={() => toggle(id)}
        onSelect={() => select({ kind: "schema", fullName })}
        action={
          <RowMenu
            label={`Schema ${schema} actions`}
            items={[
              {
                label: "Edit",
                icon: <Pencil />,
                onSelect: () =>
                  dialogs.edit({ kind: "schema", name: fullName, comment }),
              },
              {
                label: "Delete",
                icon: <Trash2 />,
                variant: "destructive",
                separatorBefore: true,
                onSelect: () =>
                  dialogs.remove({ kind: "schema", name: fullName }),
              },
            ]}
          />
        }
      />
      {isOpen(id) &&
        GROUPS.map((group) => (
          <GroupNode
            key={group.kind}
            group={group}
            catalog={catalog}
            schema={schema}
          />
        ))}
    </div>
  );
}

function GroupNode({
  group,
  catalog,
  schema,
}: {
  group: GroupDef;
  catalog: string;
  schema: string;
}) {
  const { isOpen, toggle } = useExpansion();
  const { selection, schemaTab, selectSchemaChild } = useCatalogSelection();
  const dialogs = useCatalogDialogs();
  const id = nodeId.group(catalog, schema, group.kind);
  const fullName = `${catalog}.${schema}`;
  // Subscribe to the same list query the filter bar / expanded list use (shared
  // cache key → one request, deduped). Surfaces the child count on the row once
  // loaded; a trailing "+" marks an unfetched next page.
  const { data, hasNextPage } = group.useList(catalog, schema);
  const count =
    data === undefined ? undefined : `${data.length}${hasNextPage ? "+" : ""}`;
  // A kind row is selected when its schema is shown on this kind's tab — so
  // clicking here and switching tabs in SchemaDetail cross-highlight.
  const selected =
    selection?.kind === "schema" &&
    selection.fullName === fullName &&
    schemaTab === group.kind;

  return (
    <div>
      <TreeRow
        depth={2}
        icon={<group.Icon className="h-4 w-4 text-muted-foreground" />}
        label={group.title}
        count={count}
        expandable
        open={isOpen(id)}
        selected={selected}
        onToggle={() => toggle(id)}
        onSelect={() => selectSchemaChild(fullName, group.kind)}
        action={
          group.creatable ? (
            <CreateAction
              title={`New ${group.creatable}`}
              onClick={() =>
                dialogs.create({
                  kind: group.creatable as "volume" | "model",
                  catalog,
                  schema,
                })
              }
            />
          ) : undefined
        }
      />
      {isOpen(id) && (
        <ObjectList group={group} catalog={catalog} schema={schema} />
      )}
    </div>
  );
}

function ObjectList({
  group,
  catalog,
  schema,
}: {
  group: GroupDef;
  catalog: string;
  schema: string;
}) {
  const query = group.useList(catalog, schema);
  const { selection, select } = useCatalogSelection();
  const dialogs = useCatalogDialogs();

  return (
    <ListStates
      depth={3}
      isLoading={query.isLoading}
      error={query.error}
      isEmpty={(query.data?.length ?? 0) === 0}
      hasNextPage={query.hasNextPage}
      isFetchingNextPage={query.isFetchingNextPage}
      onLoadMore={() => query.fetchNextPage()}
    >
      {query.data?.map((item) => {
        const fullName =
          ("full_name" in item && item.full_name) || objectFullName(item);
        const isSelected =
          selection?.kind === group.kind && selection.fullName === fullName;
        const editable = group.kind === "volume" || group.kind === "model";
        return (
          <TreeRow
            key={fullName || item.name}
            depth={3}
            icon={<group.Icon className="h-4 w-4 text-muted-foreground" />}
            label={item.name ?? fullName}
            selected={isSelected}
            onSelect={() => select({ kind: group.kind, fullName })}
            action={
              <RowMenu
                label={`${item.name} actions`}
                items={[
                  ...(editable
                    ? [
                        {
                          label: "Edit",
                          icon: <Pencil />,
                          onSelect: () =>
                            dialogs.edit({
                              kind: group.kind as "volume" | "model",
                              name: fullName,
                              comment: item.comment,
                            }),
                        },
                      ]
                    : []),
                  {
                    label: "Delete",
                    icon: <Trash2 />,
                    variant: "destructive" as const,
                    separatorBefore: editable,
                    onSelect: () =>
                      dialogs.remove({ kind: group.kind, name: fullName }),
                  },
                ]}
              />
            }
          />
        );
      })}
    </ListStates>
  );
}
