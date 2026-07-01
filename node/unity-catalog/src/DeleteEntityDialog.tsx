import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@open-lakehouse/ui-kit";
import {
  parseUcError,
  useDeleteCatalog,
  useDeleteCredential,
  useDeleteExternalLocation,
  useDeleteFunction,
  useDeleteRegisteredModel,
  useDeleteSchema,
  useDeleteTable,
  useDeleteVolume,
} from "@open-lakehouse/unity-catalog-client";
import { AlertTriangle } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import type { DeleteRequest } from "./dialog-types";
import { useCatalogSelection } from "./selection";

// Catalogs / schemas / models accept a `force` flag to delete when non-empty.
const FORCEABLE = new Set<DeleteRequest["kind"]>([
  "catalog",
  "schema",
  "model",
]);

export function DeleteEntityDialog({
  request,
  onClose,
}: {
  request: DeleteRequest;
  onClose: () => void;
}) {
  const { selection, select } = useCatalogSelection();
  const deleteCatalog = useDeleteCatalog();
  const deleteSchema = useDeleteSchema();
  const deleteTable = useDeleteTable();
  const deleteVolume = useDeleteVolume();
  const deleteFunction = useDeleteFunction();
  const deleteModel = useDeleteRegisteredModel();
  const deleteCredential = useDeleteCredential();
  const deleteExternalLocation = useDeleteExternalLocation();

  const [force, setForce] = useState(false);

  const pending =
    deleteCatalog.isPending ||
    deleteSchema.isPending ||
    deleteTable.isPending ||
    deleteVolume.isPending ||
    deleteFunction.isPending ||
    deleteModel.isPending ||
    deleteCredential.isPending ||
    deleteExternalLocation.isPending;

  function confirm() {
    const handlers = {
      onSuccess: () => {
        toast.success(`Deleted ${request.kind} "${request.name}"`);
        if (selection && selection.fullName === request.name) {
          select(undefined);
        }
        onClose();
      },
      onError: (error: unknown) => toast.error(parseUcError(error)),
    };

    switch (request.kind) {
      case "catalog":
        deleteCatalog.mutate(
          { params: { path: { name: request.name }, query: { force } } },
          handlers,
        );
        break;
      case "schema":
        deleteSchema.mutate(
          { params: { path: { full_name: request.name }, query: { force } } },
          handlers,
        );
        break;
      case "table":
        deleteTable.mutate(
          { params: { path: { full_name: request.name } } },
          handlers,
        );
        break;
      case "volume":
        deleteVolume.mutate(
          { params: { path: { name: request.name } } },
          handlers,
        );
        break;
      case "function":
        deleteFunction.mutate(
          { params: { path: { name: request.name } } },
          handlers,
        );
        break;
      case "model":
        deleteModel.mutate(
          { params: { path: { full_name: request.name }, query: { force } } },
          handlers,
        );
        break;
      case "credential":
        deleteCredential.mutate(
          { params: { path: { name: request.name } } },
          handlers,
        );
        break;
      case "external_location":
        deleteExternalLocation.mutate(
          { params: { path: { name: request.name } } },
          handlers,
        );
        break;
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-destructive" />
            Delete {request.kind}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-3 px-5 py-4 text-sm">
          <p>
            Are you sure you want to delete{" "}
            <span className="font-mono font-medium">{request.name}</span>? This
            action cannot be undone.
          </p>
          {FORCEABLE.has(request.kind) && (
            <label className="flex items-center gap-2 text-xs text-muted-foreground">
              <input
                type="checkbox"
                checked={force}
                onChange={(e) => setForce(e.target.checked)}
              />
              Force delete even if not empty
            </label>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="ghost" size="sm" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            size="sm"
            disabled={pending}
            onClick={confirm}
          >
            {pending ? "Deleting…" : "Delete"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
