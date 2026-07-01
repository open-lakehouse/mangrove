import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from "@open-lakehouse/ui-kit";
import {
  parseUcError,
  useUpdateCatalog,
  useUpdateRegisteredModel,
  useUpdateSchema,
  useUpdateVolume,
} from "@open-lakehouse/unity-catalog-client";
import { useId, useState } from "react";
import { toast } from "sonner";
import type { EditRequest } from "./dialog-types";
import { useCatalogSelection } from "./selection";

const TITLES: Record<EditRequest["kind"], string> = {
  catalog: "Edit catalog",
  schema: "Edit schema",
  volume: "Edit volume",
  model: "Edit registered model",
};

function renameFullName(fullName: string, newName: string) {
  const parts = fullName.split(".");
  parts[parts.length - 1] = newName;
  return parts.join(".");
}

export function EditEntityDialog({
  request,
  onClose,
}: {
  request: EditRequest;
  onClose: () => void;
}) {
  const { selection, select } = useCatalogSelection();
  const updateCatalog = useUpdateCatalog();
  const updateSchema = useUpdateSchema();
  const updateVolume = useUpdateVolume();
  const updateModel = useUpdateRegisteredModel();

  const currentName = request.name.split(".").pop() ?? request.name;
  const [newName, setNewName] = useState(currentName);
  const [comment, setComment] = useState(request.comment ?? "");

  const nameId = useId();
  const commentId = useId();

  const pending =
    updateCatalog.isPending ||
    updateSchema.isPending ||
    updateVolume.isPending ||
    updateModel.isPending;

  function finish(renamed: boolean) {
    toast.success(`Updated ${request.kind} "${currentName}"`);
    // Keep the URL selection valid if the renamed node was the selected one.
    if (renamed && selection?.fullName === request.name) {
      select({
        kind: request.kind,
        fullName: renameFullName(request.name, newName),
      });
    }
    onClose();
  }

  function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!newName.trim()) return;

    const renamed = newName !== currentName;
    const body = {
      comment: comment || undefined,
      new_name: renamed ? newName : undefined,
    };
    const handlers = {
      onSuccess: () => finish(renamed),
      onError: (error: unknown) => toast.error(parseUcError(error)),
    };

    if (request.kind === "catalog") {
      updateCatalog.mutate(
        { params: { path: { name: request.name } }, body },
        handlers,
      );
    } else if (request.kind === "schema") {
      updateSchema.mutate(
        { params: { path: { full_name: request.name } }, body },
        handlers,
      );
    } else if (request.kind === "volume") {
      updateVolume.mutate(
        { params: { path: { name: request.name } }, body },
        handlers,
      );
    } else {
      updateModel.mutate(
        { params: { path: { full_name: request.name } }, body },
        handlers,
      );
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <form onSubmit={submit}>
          <DialogHeader>
            <DialogTitle>{TITLES[request.kind]}</DialogTitle>
          </DialogHeader>

          <div className="space-y-3 px-5 py-4">
            <p className="text-xs text-muted-foreground">
              <span className="font-mono">{request.name}</span>
            </p>

            <div className="space-y-1">
              <Label htmlFor={nameId}>Name</Label>
              <Input
                id={nameId}
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                autoFocus
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor={commentId}>Comment</Label>
              <Input
                id={commentId}
                value={comment}
                onChange={(e) => setComment(e.target.value)}
                placeholder="Description"
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="ghost" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={pending || !newName.trim()}
            >
              {pending ? "Saving…" : "Save"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
