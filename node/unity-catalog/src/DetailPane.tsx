import { cn } from "@open-lakehouse/ui-kit";
import { Database, FolderTree, Globe, KeyRound } from "lucide-react";
import type { ReactNode } from "react";
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
import { PANE_HEADER_CLASS } from "./layout";
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
  const { selection } = useCatalogSelection();

  if (!selection) {
    return (
      <div className="flex min-h-0 items-center justify-center p-8 text-center text-sm text-muted-foreground">
        Select an object from the tree to see its details.
      </div>
    );
  }

  const { object } = splitFullName(selection.fullName);
  const displayName = selection.fullName || object;

  return (
    <div className="flex min-h-0 flex-col overflow-auto">
      <div
        className={cn(
          PANE_HEADER_CLASS,
          "sticky top-0 z-10 bg-background px-6",
        )}
      >
        <div className="group flex min-w-0 items-center gap-2.5">
          {detailIcon(selection.kind)}
          <span className="truncate font-mono text-base font-semibold">
            {displayName}
          </span>
          {displayName && <CopyButton value={displayName} label="full name" />}
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
