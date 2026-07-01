import { Button } from "@open-lakehouse/ui-kit";
import {
  objectFullName,
  prefetchSchemas,
  useCatalogs,
  useSchemas,
} from "@open-lakehouse/unity-catalog-client";
import { useQueryClient } from "@tanstack/react-query";
import { Database, FolderTree, Pencil, Plus, Trash2 } from "lucide-react";
import { useCatalogDialogs } from "./dialogs";
import { nodeId, useExpansion } from "./ExpansionContext";
import { GROUPS, type GroupDef } from "./groups";
import { RowMenu } from "./RowMenu";
import { useCatalogSelection } from "./selection";
import { CreateAction, ListStates, TreeRow } from "./TreeRow";

export function CatalogTree() {
  const queryClient = useQueryClient();
  const catalogs = useCatalogs();
  const dialogs = useCatalogDialogs();

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between border-b px-3 py-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        <span className="flex items-center gap-2">
          <Database className="h-4 w-4" />
          Catalogs
        </span>
        <Button
          variant="ghost"
          size="sm"
          className="h-6 px-1.5 text-xs"
          onClick={() => dialogs.create({ kind: "catalog" })}
        >
          <Plus className="h-3.5 w-3.5" />
          New
        </Button>
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
  const { selection, select } = useCatalogSelection();
  const dialogs = useCatalogDialogs();
  const id = nodeId.schema(catalog, schema);
  const fullName = `${catalog}.${schema}`;
  const selected =
    selection?.kind === "schema" && selection.fullName === fullName;

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
  const dialogs = useCatalogDialogs();
  const id = nodeId.group(catalog, schema, group.kind);

  return (
    <div>
      <TreeRow
        depth={2}
        icon={<group.Icon className="h-4 w-4 text-muted-foreground" />}
        label={group.title}
        expandable
        open={isOpen(id)}
        onToggle={() => toggle(id)}
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
