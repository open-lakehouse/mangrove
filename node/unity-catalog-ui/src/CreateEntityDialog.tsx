// Create dialog for the catalog namespace: routes a create request to the
// resource's form. Parent catalog/schema may still be missing (the global
// "New …" menu), so each form has an inline parent picker and blocks submit
// until the parent is chosen.

import type { CatalogCreateRequest } from "./dialog-types.js";
import { CatalogCreateDialog } from "./forms/resources/catalog.js";
import {
  ModelCreateDialog,
  VolumeCreateDialog,
} from "./forms/resources/leaf.js";
import { SchemaCreateDialog } from "./forms/resources/schema.js";

export function CreateEntityDialog({
  request,
  onClose,
}: {
  request: CatalogCreateRequest;
  onClose: () => void;
}) {
  switch (request.kind) {
    case "catalog":
      return <CatalogCreateDialog onClose={onClose} />;
    case "schema":
      return <SchemaCreateDialog catalog={request.catalog} onClose={onClose} />;
    case "volume":
      return (
        <VolumeCreateDialog
          catalog={request.catalog}
          schema={request.schema}
          onClose={onClose}
        />
      );
    case "model":
      return (
        <ModelCreateDialog
          catalog={request.catalog}
          schema={request.schema}
          onClose={onClose}
        />
      );
  }
}
