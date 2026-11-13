// Edit dialog for the catalog namespace: routes an edit request to the
// resource's form. Each form loads the current object, so it edits what the
// server has rather than what the tree row last showed.

import type { EditRequest } from "./dialog-types.js";
import { CatalogEditDialog } from "./forms/resources/catalog.js";
import { ModelEditDialog, VolumeEditDialog } from "./forms/resources/leaf.js";
import { SchemaEditDialog } from "./forms/resources/schema.js";

export function EditEntityDialog({
  request,
  onClose,
}: {
  request: EditRequest;
  onClose: () => void;
}) {
  switch (request.kind) {
    case "catalog":
      return <CatalogEditDialog name={request.name} onClose={onClose} />;
    case "schema":
      return <SchemaEditDialog fullName={request.name} onClose={onClose} />;
    case "volume":
      return <VolumeEditDialog fullName={request.name} onClose={onClose} />;
    case "model":
      return <ModelEditDialog fullName={request.name} onClose={onClose} />;
  }
}
