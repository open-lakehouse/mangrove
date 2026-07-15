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
import type { VolumeType } from "@open-lakehouse/unity-catalog-client";
import {
  parseUcError,
  useCreateCatalog,
  useCreateRegisteredModel,
  useCreateSchema,
  useCreateVolume,
} from "@open-lakehouse/unity-catalog-client";
import type { RJSFSchema, UiSchema } from "@rjsf/utils";
import { useId, useMemo, useState } from "react";
import { toast } from "sonner";
import { CatalogSchemaPicker } from "./CatalogSchemaPicker";
import { useRevealCreated } from "./create-reconcile";
import type { CatalogCreateRequest } from "./dialog-types";
import { SchemaForm } from "./forms/SchemaForm";
import { cloneSchema, formSchemas } from "./forms/schemas";
import { StorageLocationPicker } from "./storage/StorageLocationPicker";

const TITLES: Record<CatalogCreateRequest["kind"], string> = {
  catalog: "New catalog",
  schema: "New schema",
  volume: "New volume",
  model: "New registered model",
};

// The kind is always known when a create dialog opens (chosen from a dropdown or
// implied by a tree location). Parent catalog/schema may still be missing, so
// the securable form is shown immediately with an inline picker; the Create
// button stays disabled until any required parent context is chosen.
export function CreateEntityDialog({
  request,
  onClose,
}: {
  request: CatalogCreateRequest;
  onClose: () => void;
}) {
  if (request.kind === "catalog" || request.kind === "schema") {
    return <NamespaceCreateDialog request={request} onClose={onClose} />;
  }
  return <LeafCreateDialog request={request} onClose={onClose} />;
}

const NAMESPACE_FORM_ID = "namespace-create-form";

type LooseSchema = Record<string, unknown>;

interface NamespaceFormData {
  name?: string;
  catalog_name?: string;
  comment?: string;
}

/** Reduce a catalog/schema create schema to the name + comment fields. */
function tailorNamespaceSchema(base: RJSFSchema): RJSFSchema {
  const schema = cloneSchema(base);
  const props = (schema.properties ?? {}) as Record<string, LooseSchema>;
  // The key-value `properties` map is an object field; drop it (not surfaced).
  delete props.properties;
  return schema;
}

// Hidden fields are still part of the schema (so injected values validate) but
// not rendered; storage is handled by the dedicated picker below the form.
const NAMESPACE_HIDDEN_UI: UiSchema = {
  catalog_name: { "ui:widget": "hidden" },
  storage_root: { "ui:widget": "hidden" },
  storage_location: { "ui:widget": "hidden" },
  provider_name: { "ui:widget": "hidden" },
  share_name: { "ui:widget": "hidden" },
  comment: { "ui:placeholder": "Description (optional)" },
  name: { "ui:placeholder": "my_object", "ui:autofocus": true },
  "ui:order": ["name", "comment", "*"],
};

function NamespaceCreateDialog({
  request,
  onClose,
}: {
  request: Extract<CatalogCreateRequest, { kind: "catalog" | "schema" }>;
  onClose: () => void;
}) {
  const createCatalog = useCreateCatalog();
  const createSchema = useCreateSchema();
  const reveal = useRevealCreated();

  const isSchema = request.kind === "schema";

  const [catalog, setCatalog] = useState(() =>
    isSchema ? (request.catalog ?? "") : "",
  );
  const [formData, setFormData] = useState<NamespaceFormData>({});
  const [storageRoot, setStorageRoot] = useState<string>();

  const schema = useMemo(
    () =>
      tailorNamespaceSchema(
        request.kind === "catalog"
          ? formSchemas.createCatalog
          : formSchemas.createSchema,
      ),
    [request.kind],
  );

  const pending = createCatalog.isPending || createSchema.isPending;
  const canSubmit = !isSchema || !!catalog;

  function submit(data: NamespaceFormData) {
    if (isSchema && !catalog) return;

    const handlers = {
      onSuccess: () => {
        const createdName = data.name ?? "";
        toast.success(`Created ${request.kind} "${createdName}"`);
        if (request.kind === "catalog") {
          reveal({ kind: "catalog", name: createdName });
        } else {
          reveal({ kind: "schema", name: createdName, catalog });
        }
        onClose();
      },
      onError: (error: unknown) => toast.error(parseUcError(error)),
    };

    if (request.kind === "catalog") {
      createCatalog.mutate(
        {
          body: {
            name: data.name ?? "",
            comment: data.comment || undefined,
            storage_root: storageRoot,
          },
        },
        handlers,
      );
      return;
    }

    createSchema.mutate(
      {
        body: {
          name: data.name ?? "",
          catalog_name: catalog,
          comment: data.comment || undefined,
          storage_root: storageRoot,
        },
      },
      handlers,
    );
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{TITLES[request.kind]}</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 px-5 py-4">
          <SchemaForm<NamespaceFormData>
            id={NAMESPACE_FORM_ID}
            schema={schema}
            uiSchema={NAMESPACE_HIDDEN_UI}
            formData={formData}
            disabled={pending}
            onChange={setFormData}
            onSubmit={submit}
          />

          <StorageLocationPicker onChange={setStorageRoot} />

          {isSchema && (
            <CatalogSchemaPicker
              catalog={catalog}
              schema=""
              onCatalogChange={setCatalog}
              onSchemaChange={() => {}}
              requireCatalog
              requireSchema={false}
            />
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="ghost" size="sm" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            form={NAMESPACE_FORM_ID}
            size="sm"
            disabled={pending || !canSubmit}
          >
            {pending ? "Creating…" : "Create"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function LeafCreateDialog({
  request,
  onClose,
}: {
  request: Extract<CatalogCreateRequest, { kind: "volume" | "model" }>;
  onClose: () => void;
}) {
  const createVolume = useCreateVolume();
  const createModel = useCreateRegisteredModel();
  const reveal = useRevealCreated();

  const [name, setName] = useState("");
  const [comment, setComment] = useState("");
  const [volumeType, setVolumeType] = useState<VolumeType>("MANAGED");
  const [storageLocation, setStorageLocation] = useState("");
  const [catalog, setCatalog] = useState(request.catalog ?? "");
  const [schema, setSchema] = useState(request.schema ?? "");

  const nameId = useId();
  const volumeTypeId = useId();
  const storageLocationId = useId();
  const commentId = useId();

  const pending = createVolume.isPending || createModel.isPending;
  const canSubmit = !!name.trim() && !!catalog && !!schema;

  function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!canSubmit) return;

    const handlers = {
      onSuccess: () => {
        toast.success(`Created ${request.kind} "${name}"`);
        reveal({ kind: request.kind, name, catalog, schema });
        onClose();
      },
      onError: (error: unknown) => toast.error(parseUcError(error)),
    };

    if (request.kind === "volume") {
      createVolume.mutate(
        {
          body: {
            name,
            catalog_name: catalog,
            schema_name: schema,
            volume_type: volumeType,
            comment: comment || undefined,
            storage_location:
              volumeType === "EXTERNAL" ? storageLocation : undefined,
          },
        },
        handlers,
      );
    } else {
      createModel.mutate(
        {
          body: {
            name,
            catalog_name: catalog,
            schema_name: schema,
            comment: comment || undefined,
          },
        },
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

          <div className="space-y-4 px-5 py-4">
            <div className="space-y-1">
              <Label htmlFor={nameId}>Name</Label>
              <Input
                id={nameId}
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="my_object"
                autoFocus
              />
            </div>

            {request.kind === "volume" && (
              <>
                <div className="space-y-1">
                  <Label htmlFor={volumeTypeId}>Volume type</Label>
                  <select
                    id={volumeTypeId}
                    value={volumeType}
                    onChange={(e) =>
                      setVolumeType(e.target.value as VolumeType)
                    }
                    className="flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <option value="MANAGED">MANAGED</option>
                    <option value="EXTERNAL">EXTERNAL</option>
                  </select>
                </div>
                {volumeType === "EXTERNAL" && (
                  <div className="space-y-1">
                    <Label htmlFor={storageLocationId}>Storage location</Label>
                    <Input
                      id={storageLocationId}
                      value={storageLocation}
                      onChange={(e) => setStorageLocation(e.target.value)}
                      placeholder="s3://bucket/path"
                    />
                  </div>
                )}
              </>
            )}

            <div className="space-y-1">
              <Label htmlFor={commentId}>Comment (optional)</Label>
              <Input
                id={commentId}
                value={comment}
                onChange={(e) => setComment(e.target.value)}
                placeholder="Description"
              />
            </div>

            <CatalogSchemaPicker
              catalog={catalog}
              schema={schema}
              onCatalogChange={setCatalog}
              onSchemaChange={setSchema}
              requireCatalog
              requireSchema
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="ghost" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" size="sm" disabled={pending || !canSubmit}>
              {pending ? "Creating…" : "Create"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
