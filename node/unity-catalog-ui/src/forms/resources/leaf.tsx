// Create and edit the schema-scoped securables that need no data experience:
// volumes and registered models. Tables get their own flow.

import type { components } from "@open-lakehouse/unity-catalog-client";
import {
  useCreateRegisteredModel,
  useCreateVolume,
  useModelDetail,
  useUpdateRegisteredModel,
  useUpdateVolume,
  useVolumeDetail,
} from "@open-lakehouse/unity-catalog-client/react";
import { useEffect } from "react";
import { toast } from "sonner";
import { CatalogSchemaPicker } from "../../CatalogSchemaPicker.js";
import { useRevealCreated } from "../../create-reconcile.js";
import { CommentField, EnumField, NameField, TextField } from "../fields.js";
import { ResourceFormDialog } from "../ResourceFormDialog.js";
import {
  createRegisteredModel,
  createVolume,
  updateRegisteredModel,
  updateVolume,
} from "../request-fields.gen.js";
import type { FieldErrors } from "../spec.js";
import type { ResourceForm } from "../useResourceForm.js";
import { useResourceForm } from "../useResourceForm.js";
import {
  createBody,
  leafName,
  loadErrorOf,
  requireNewName,
  useFollowRename,
} from "./shared.js";

type Schemas = components["schemas"];
type CreateVolume = Schemas["CreateVolumeRequestContent"];
type UpdateVolume = Schemas["UpdateVolumeRequest"];
type CreateModel = Schemas["CreateRegisteredModel"];
type UpdateModel = Schemas["UpdateRegisteredModelRequest"];

/** The parent catalog + schema pickers, bound to a leaf create form. */
function ParentPicker({
  form,
}: {
  form: ResourceForm<{ catalog_name?: string; schema_name?: string }>;
}) {
  const error = form.error("catalog_name") ?? form.error("schema_name");
  return (
    <div className="space-y-1.5">
      <CatalogSchemaPicker
        catalog={form.values.catalog_name ?? ""}
        schema={form.values.schema_name ?? ""}
        onCatalogChange={(next) => {
          form.set("catalog_name", next || undefined);
          form.set("schema_name", undefined);
        }}
        onSchemaChange={(next) => form.set("schema_name", next || undefined)}
        requireCatalog
        requireSchema
      />
      {error && <p className="text-xs text-destructive">{error}</p>}
    </div>
  );
}

const VOLUME_TYPE_LABELS: Record<string, string> = {
  MANAGED: "Managed: stored in the schema's managed storage",
  EXTERNAL: "External: stored at a location you choose",
};

export function VolumeCreateDialog({
  catalog,
  schema,
  onClose,
}: {
  catalog?: string;
  schema?: string;
  onClose: () => void;
}) {
  const create = useCreateVolume();
  const reveal = useRevealCreated();
  const form = useResourceForm<CreateVolume>(
    createVolume,
    { catalog_name: catalog, schema_name: schema, volume_type: "MANAGED" },
    {
      validate: (values): FieldErrors =>
        values.volume_type === "EXTERNAL" && !values.storage_location
          ? { storage_location: "Required for an external volume." }
          : {},
    },
  );
  const external = form.values.volume_type === "EXTERNAL";

  const submit = form.submit((values) => {
    const body = createBody<CreateVolume>(values);
    if (body.volume_type !== "EXTERNAL") delete body.storage_location;
    create.mutate(
      { body },
      {
        onSuccess: () => {
          toast.success(`Created volume "${body.name}"`);
          reveal({
            kind: "volume",
            name: body.name,
            catalog: body.catalog_name,
            schema: body.schema_name,
          });
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="New volume"
      description="A volume holds non-tabular files in a schema."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <ParentPicker form={form} />
      <NameField form={form} name="name" placeholder="raw_files" autoFocus />
      <EnumField
        form={form}
        name="volume_type"
        label="Volume type"
        help={null}
        labels={VOLUME_TYPE_LABELS}
      />
      {external && (
        <TextField
          form={form}
          name="storage_location"
          label="Storage location"
          mono
          placeholder="s3://bucket/path"
          help="Must lie inside a registered external location."
          required
        />
      )}
      <CommentField form={form} />
    </ResourceFormDialog>
  );
}

export function ModelCreateDialog({
  catalog,
  schema,
  onClose,
}: {
  catalog?: string;
  schema?: string;
  onClose: () => void;
}) {
  const create = useCreateRegisteredModel();
  const reveal = useRevealCreated();
  const form = useResourceForm<CreateModel>(createRegisteredModel, {
    catalog_name: catalog,
    schema_name: schema,
  });

  const submit = form.submit((values) => {
    const body = createBody<CreateModel>(values);
    create.mutate(
      { body },
      {
        onSuccess: () => {
          toast.success(`Created registered model "${body.name}"`);
          reveal({
            kind: "model",
            name: body.name,
            catalog: body.catalog_name,
            schema: body.schema_name,
          });
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="New registered model"
      description="A registered model groups the versions of an ML model."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <ParentPicker form={form} />
      <NameField form={form} name="name" placeholder="churn_model" autoFocus />
      <CommentField form={form} />
    </ResourceFormDialog>
  );
}

/** Rename / re-comment a volume or registered model. */
function LeafEditDialog({
  kind,
  fullName,
  loaded,
  loadError,
  pending,
  onSave,
  onClose,
}: {
  kind: "volume" | "model";
  fullName: string;
  loaded: { name?: string; comment?: string } | undefined;
  loadError?: string;
  pending: boolean;
  onSave: (
    body: { new_name?: string; comment?: string },
    done: () => void,
    fail: (error: unknown) => void,
  ) => void;
  onClose: () => void;
}) {
  const followRename = useFollowRename(kind, fullName);
  const specs = kind === "volume" ? updateVolume : updateRegisteredModel;
  const form = useResourceForm<UpdateVolume & UpdateModel>(
    specs,
    {},
    { validate: requireNewName, patch: true },
  );

  const { seed } = form;
  useEffect(() => {
    if (loaded) {
      seed({
        new_name: loaded.name ?? leafName(fullName),
        comment: loaded.comment,
      });
    }
  }, [loaded, seed, fullName]);

  const submit = form.submit(() => {
    const body = form.changes(["new_name", "comment"]);
    if (Object.keys(body).length === 0) return onClose();
    if (body.new_name) body.new_name = body.new_name.trim();
    onSave(
      body,
      () => {
        const label = kind === "volume" ? "volume" : "registered model";
        toast.success(
          `Updated ${label} "${body.new_name ?? leafName(fullName)}"`,
        );
        followRename(body.new_name);
        onClose();
      },
      form.fail,
    );
  });

  return (
    <ResourceFormDialog
      title={kind === "volume" ? "Edit volume" : "Edit registered model"}
      description={<span className="font-mono">{fullName}</span>}
      submitLabel="Save"
      pending={pending}
      canSubmit={!!loaded}
      loadError={loadError}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <NameField
        form={form}
        name="new_name"
        label="Name"
        disabled={!loaded}
        autoFocus
      />
      <CommentField form={form} disabled={!loaded} />
    </ResourceFormDialog>
  );
}

export function VolumeEditDialog({
  fullName,
  onClose,
}: {
  fullName: string;
  onClose: () => void;
}) {
  const update = useUpdateVolume();
  const detail = useVolumeDetail(fullName);
  return (
    <LeafEditDialog
      kind="volume"
      fullName={fullName}
      loaded={detail.data}
      loadError={loadErrorOf(detail)}
      pending={update.isPending}
      onSave={(body, done, fail) =>
        update.mutate(
          { params: { path: { name: fullName } }, body },
          { onSuccess: () => done(), onError: fail },
        )
      }
      onClose={onClose}
    />
  );
}

export function ModelEditDialog({
  fullName,
  onClose,
}: {
  fullName: string;
  onClose: () => void;
}) {
  const update = useUpdateRegisteredModel();
  const detail = useModelDetail(fullName);
  return (
    <LeafEditDialog
      kind="model"
      fullName={fullName}
      loaded={detail.data}
      loadError={loadErrorOf(detail)}
      pending={update.isPending}
      onSave={(body, done, fail) =>
        update.mutate(
          { params: { path: { full_name: fullName } }, body },
          { onSuccess: () => done(), onError: fail },
        )
      }
      onClose={onClose}
    />
  );
}
