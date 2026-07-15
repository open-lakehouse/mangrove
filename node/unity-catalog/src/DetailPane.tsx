import { Database, FolderTree, Globe, KeyRound } from "lucide-react";
import type { ReactNode } from "react";
import { Breadcrumbs, type Crumb } from "./Breadcrumbs";
import { CopyButton } from "./CopyButton";
import { CatalogDetail } from "./detail/CatalogDetail";
import { CredentialDetail } from "./detail/CredentialDetail";
import { ExternalLocationDetail } from "./detail/ExternalLocationDetail";
import { FunctionDetail } from "./detail/FunctionDetail";
import { ModelDetail } from "./detail/ModelDetail";
import { SchemaDetail } from "./detail/SchemaDetail";
import { TableDetail } from "./detail/TableDetail";
import { VolumeDetail } from "./detail/VolumeDetail";
import { kindIcon } from "./groups";
import { useCatalogSelection } from "./selection";
import { isObjectKind, type SelectableKind, splitFullName } from "./types";

function detailIcon(kind: SelectableKind): ReactNode {
  const cls = "h-5 w-5 shrink-0 text-muted-foreground";
  if (kind === "catalog") return <Database className={cls} />;
  if (kind === "schema") return <FolderTree className={cls} />;
  if (kind === "credential") return <KeyRound className={cls} />;
  if (kind === "external_location") return <Globe className={cls} />;
  if (isObjectKind(kind)) return kindIcon(kind, cls);
  return null;
}

export function DetailPane() {
  const { selection, select } = useCatalogSelection();

  if (!selection) {
    return (
      <div className="flex min-h-0 items-center justify-center p-8 text-center text-sm text-muted-foreground">
        Select an object from the tree to see its details.
      </div>
    );
  }

  const { catalog, schema, object } = splitFullName(selection.fullName);
  // Title shows the object's short (leaf) name — the parent path lives in the
  // breadcrumb above it, Databricks-style.
  const shortName = object ?? schema ?? catalog ?? selection.fullName;

  // Ancestors that lead to the selection, each selecting that node in the tree.
  // The metastore-level securables (catalog / credential / external location)
  // sit directly under the explorer root, so they get no extra crumbs.
  const crumbs: Crumb[] = [
    { label: "Catalog Explorer", onClick: () => select(undefined) },
  ];
  if (catalog && schema) {
    crumbs.push({
      label: catalog,
      onClick: () => select({ kind: "catalog", fullName: catalog }),
    });
  }
  if (catalog && schema && object) {
    crumbs.push({
      label: schema,
      onClick: () =>
        select({ kind: "schema", fullName: `${catalog}.${schema}` }),
    });
  }

  return (
    <div className="flex min-h-0 flex-col overflow-auto">
      <div className="sticky top-0 z-10 shrink-0 bg-background px-6 pt-3 pb-2">
        <Breadcrumbs items={crumbs} />
        <div className="group mt-1.5 flex min-w-0 items-center gap-2.5">
          {detailIcon(selection.kind)}
          <span className="truncate font-mono text-lg font-semibold">
            {shortName}
          </span>
          <CopyButton value={selection.fullName} label="full name" />
        </div>
      </div>
      <div className="px-6 py-6">
        {selection.kind === "catalog" && (
          <CatalogDetail name={selection.fullName} />
        )}
        {selection.kind === "schema" && (
          <SchemaDetail fullName={selection.fullName} />
        )}
        {selection.kind === "table" && (
          <TableDetail fullName={selection.fullName} />
        )}
        {selection.kind === "volume" && (
          <VolumeDetail fullName={selection.fullName} />
        )}
        {selection.kind === "function" && (
          <FunctionDetail fullName={selection.fullName} />
        )}
        {selection.kind === "model" && (
          <ModelDetail fullName={selection.fullName} />
        )}
        {selection.kind === "credential" && (
          <CredentialDetail name={selection.fullName} />
        )}
        {selection.kind === "external_location" && (
          <ExternalLocationDetail name={selection.fullName} />
        )}
      </div>
    </div>
  );
}
