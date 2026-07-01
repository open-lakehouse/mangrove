// Tabular admin view for a metastore-level storage securable (external locations
// or credentials). Rich rows surface the key fields inline with hover Edit/Delete
// actions; clicking a row expands its full detail in place. Create/edit/delete
// reuse the shared dialog flow (useCatalogDialogs) and the same UC list hooks the
// sidebar used, so this is a presentation change over existing plumbing.

import { Button, cn } from "@open-lakehouse/ui-kit";
import type {
  CredentialInfo,
  ExternalLocationInfo,
} from "@open-lakehouse/unity-catalog-client";
import {
  useCredentials,
  useExternalLocations,
} from "@open-lakehouse/unity-catalog-client";
import type { UseInfiniteQueryResult } from "@tanstack/react-query";
import {
  ChevronDown,
  ChevronRight,
  Globe,
  KeyRound,
  type LucideIcon,
  Pencil,
  Plus,
  Trash2,
} from "lucide-react";
import { type ReactNode, useState } from "react";
import { CredentialDetail } from "../detail/CredentialDetail";
import { ExternalLocationDetail } from "../detail/ExternalLocationDetail";
import { useCatalogDialogs } from "../dialogs";
import { RowMenu } from "../RowMenu";
import type { StorageKind } from "../types";

type StorageItem = CredentialInfo | ExternalLocationInfo;
type StorageList = UseInfiniteQueryResult<StorageItem[], unknown>;

interface Column {
  header: string;
  /** Extra value cells rendered after the always-present name cell. */
  cell: (item: StorageItem) => ReactNode;
}

interface StorageTableDef {
  kind: StorageKind;
  createLabel: string;
  emptyLabel: string;
  Icon: LucideIcon;
  useList: () => StorageList;
  columns: Column[];
  Detail: (props: { name: string }) => ReactNode;
}

const DEFS: Record<StorageKind, StorageTableDef> = {
  external_location: {
    kind: "external_location",
    createLabel: "New external location",
    emptyLabel: "No external locations yet.",
    Icon: Globe,
    useList: useExternalLocations as () => StorageList,
    columns: [
      {
        header: "URL",
        cell: (item) => (
          <span className="font-mono text-xs">
            {(item as ExternalLocationInfo).url}
          </span>
        ),
      },
      {
        header: "Credential",
        cell: (item) => (item as ExternalLocationInfo).credential_name,
      },
      { header: "Owner", cell: (item) => item.owner },
    ],
    Detail: ExternalLocationDetail,
  },
  credential: {
    kind: "credential",
    createLabel: "New credential",
    emptyLabel: "No credentials yet.",
    Icon: KeyRound,
    useList: useCredentials as () => StorageList,
    columns: [
      { header: "Purpose", cell: (item) => (item as CredentialInfo).purpose },
      { header: "Owner", cell: (item) => item.owner },
      { header: "Created by", cell: (item) => item.created_by },
    ],
    Detail: CredentialDetail,
  },
};

export function StorageTable({ kind }: { kind: StorageKind }) {
  const def = DEFS[kind];
  const query = def.useList();
  const dialogs = useCatalogDialogs();
  const [expanded, setExpanded] = useState<string | null>(null);

  const items = query.data ?? [];

  return (
    <div className="flex min-h-0 flex-col">
      <div className="flex items-center justify-end border-b px-3 py-2">
        <Button size="sm" onClick={() => dialogs.create({ kind: def.kind })}>
          <Plus className="h-4 w-4" />
          {def.createLabel}
        </Button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {query.isLoading ? (
          <p className="px-3 py-6 text-sm text-muted-foreground">Loading…</p>
        ) : query.error ? (
          <p className="px-3 py-6 text-sm text-destructive">
            {query.error instanceof Error
              ? query.error.message
              : "Failed to load."}
          </p>
        ) : items.length === 0 ? (
          <p className="px-3 py-6 text-sm text-muted-foreground">
            {def.emptyLabel}
          </p>
        ) : (
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="border-b text-left text-xs font-medium uppercase tracking-wide text-muted-foreground">
                <th className="w-8" />
                <th className="px-3 py-2">Name</th>
                {def.columns.map((c) => (
                  <th key={c.header} className="px-3 py-2">
                    {c.header}
                  </th>
                ))}
                <th className="w-16 px-3 py-2 text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => {
                const name = item.name ?? "";
                const isOpen = expanded === name;
                return (
                  <StorageRow
                    key={name}
                    def={def}
                    item={item}
                    name={name}
                    isOpen={isOpen}
                    onToggle={() =>
                      setExpanded((cur) => (cur === name ? null : name))
                    }
                    onEdit={() => dialogs.edit({ kind: def.kind, name })}
                    onDelete={() => dialogs.remove({ kind: def.kind, name })}
                  />
                );
              })}
            </tbody>
          </table>
        )}

        {query.hasNextPage ? (
          <div className="px-3 py-2">
            <Button
              variant="ghost"
              size="sm"
              disabled={query.isFetchingNextPage}
              onClick={() => query.fetchNextPage()}
            >
              {query.isFetchingNextPage ? "Loading…" : "Load more"}
            </Button>
          </div>
        ) : null}
      </div>
    </div>
  );
}

function StorageRow({
  def,
  item,
  name,
  isOpen,
  onToggle,
  onEdit,
  onDelete,
}: {
  def: StorageTableDef;
  item: StorageItem;
  name: string;
  isOpen: boolean;
  onToggle: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { Detail } = def;
  const colSpan = def.columns.length + 3;

  return (
    <>
      <tr
        className={cn(
          "group cursor-pointer border-b hover:bg-accent/40",
          isOpen && "bg-accent/40",
        )}
        onClick={onToggle}
      >
        <td className="pl-3 text-muted-foreground">
          {isOpen ? (
            <ChevronDown className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </td>
        <td className="px-3 py-2">
          <span className="flex items-center gap-2 font-medium">
            <def.Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
            {name}
          </span>
        </td>
        {def.columns.map((c) => (
          <td key={c.header} className="px-3 py-2 text-muted-foreground">
            {c.cell(item)}
          </td>
        ))}
        <td className="px-3 py-2 text-right">
          <div className="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 focus-within:opacity-100">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0"
              aria-label={`Edit ${name}`}
              title="Edit"
              onClick={(e) => {
                e.stopPropagation();
                onEdit();
              }}
            >
              <Pencil className="h-3.5 w-3.5" />
            </Button>
            <RowMenu
              label={`${name} actions`}
              items={[
                { label: "Edit", icon: <Pencil />, onSelect: onEdit },
                {
                  label: "Delete",
                  icon: <Trash2 />,
                  variant: "destructive",
                  separatorBefore: true,
                  onSelect: onDelete,
                },
              ]}
            />
          </div>
        </td>
      </tr>
      {isOpen ? (
        <tr className="border-b bg-muted/20">
          <td colSpan={colSpan} className="px-6 py-4">
            <Detail name={name} />
          </td>
        </tr>
      ) : null}
    </>
  );
}
