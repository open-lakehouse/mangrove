import { Button, cn } from "@open-lakehouse/ui-kit";
import {
  Database,
  FolderTree,
  Globe,
  KeyRound,
  Pencil,
  Trash2,
  X,
} from "lucide-react";
import type { ReactNode } from "react";
import { CatalogDetail } from "./detail/CatalogDetail";
import { CredentialDetail } from "./detail/CredentialDetail";
import { ExternalLocationDetail } from "./detail/ExternalLocationDetail";
import { FunctionDetail } from "./detail/FunctionDetail";
import { ModelDetail } from "./detail/ModelDetail";
import { SchemaDetail } from "./detail/SchemaDetail";
import { TableDetail, TableHeaderMeta } from "./detail/TableDetail";
import { VolumeDetail, VolumeHeaderMeta } from "./detail/VolumeDetail";
import type { AnyEditRequest } from "./dialog-types";
import { useCatalogDialogs } from "./dialogs";
import { kindIcon } from "./groups";
import { PANE_HEADER_CLASS } from "./layout";
import { useCatalogSelection } from "./selection";
import { isObjectKind, type SelectableKind, splitFullName } from "./types";

// Catalogs / schemas / volumes / models / credentials / external locations
// support PATCH; tables / functions don't.
const EDITABLE: ReadonlySet<SelectableKind> = new Set<SelectableKind>([
  "catalog",
  "schema",
  "volume",
  "model",
  "credential",
  "external_location",
]);

function detailIcon(kind: SelectableKind): ReactNode {
  if (kind === "catalog")
    return <Database className="h-4 w-4 text-muted-foreground" />;
  if (kind === "schema")
    return <FolderTree className="h-4 w-4 text-muted-foreground" />;
  if (kind === "credential")
    return <KeyRound className="h-4 w-4 text-muted-foreground" />;
  if (kind === "external_location")
    return <Globe className="h-4 w-4 text-muted-foreground" />;
  if (isObjectKind(kind))
    return kindIcon(kind, "h-4 w-4 text-muted-foreground");
  return null;
}

export function DetailPane() {
  const { selection, select } = useCatalogSelection();
  const dialogs = useCatalogDialogs();

  if (!selection) {
    return (
      <div className="flex min-h-0 items-center justify-center p-8 text-center text-sm text-muted-foreground">
        Select an object from the tree to see its details.
      </div>
    );
  }

  const { object } = splitFullName(selection.fullName);
  const editable = EDITABLE.has(selection.kind);

  return (
    <div className="flex min-h-0 flex-col overflow-auto">
      <div className={cn(PANE_HEADER_CLASS, "sticky top-0 z-10 bg-card px-6")}>
        <div className="flex min-w-0 items-center gap-2">
          {detailIcon(selection.kind)}
          <span className="truncate font-mono text-sm font-medium">
            {selection.fullName || object}
          </span>
          {selection.kind === "table" && (
            <TableHeaderMeta fullName={selection.fullName} />
          )}
          {selection.kind === "volume" && (
            <VolumeHeaderMeta fullName={selection.fullName} />
          )}
        </div>
        <div className="flex items-center gap-1">
          {editable && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7"
              onClick={() =>
                dialogs.edit({
                  kind: selection.kind,
                  name: selection.fullName,
                } as AnyEditRequest)
              }
            >
              <Pencil className="h-3.5 w-3.5" />
              Edit
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-destructive hover:text-destructive"
            onClick={() =>
              dialogs.remove({ kind: selection.kind, name: selection.fullName })
            }
          >
            <Trash2 className="h-3.5 w-3.5" />
            Delete
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            aria-label="Close"
            onClick={() => select(undefined)}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>
      <div className="px-6 py-4">
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
