// Create and edit a schema.

import type { components } from "@open-lakehouse/unity-catalog-client";
import {
  useCreateSchema,
  useSchemaDetail,
  useUpdateSchema,
} from "@open-lakehouse/unity-catalog-client/react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { CatalogSchemaPicker } from "../../CatalogSchemaPicker.js";
import { useRevealCreated } from "../../create-reconcile.js";
import { StorageLocationPicker } from "../../storage/StorageLocationPicker.js";
import { CommentField, NameField, PropertiesField } from "../fields.js";
import { ResourceFormDialog } from "../ResourceFormDialog.js";
import { createSchema, updateSchema } from "../request-fields.gen.js";
import { useResourceForm } from "../useResourceForm.js";
import {
  createBody,
  leafName,
  loadErrorOf,
  requireNewName,
  STORAGE_INCOMPLETE,
  useFollowRename,
} from "./shared.js";

type CreateSchema = components["schemas"]["CreateSchema"];
type UpdateSchema = components["schemas"]["UpdateSchemaRequest"];

export function SchemaCreateDialog({
  catalog,
  onClose,
}: {
  /** Pre-selected parent catalog (e.g. when launched from the tree). */
  catalog?: string;
  onClose: () => void;
}) {
  const create = useCreateSchema();
  const reveal = useRevealCreated();
  const [storageIncomplete, setStorageIncomplete] = useState(false);

  const form = useResourceForm<CreateSchema>(
    createSchema,
    { catalog_name: catalog },
    {
      validate: () =>
        storageIncomplete ? { storage_root: STORAGE_INCOMPLETE } : {},
    },
  );

  const submit = form.submit((values) => {
    const body = createBody<CreateSchema>(values);
    create.mutate(
      { body },
      {
        onSuccess: () => {
          toast.success(`Created schema "${body.catalog_name}.${body.name}"`);
          reveal({
            kind: "schema",
            name: body.name,
            catalog: body.catalog_name,
          });
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  const catalogError = form.error("catalog_name");

  return (
    <ResourceFormDialog
      title="New schema"
      description="A schema groups tables, volumes, functions and models within a catalog."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <div className="space-y-1.5">
        <CatalogSchemaPicker
          catalog={form.values.catalog_name ?? ""}
          schema=""
          onCatalogChange={(next) =>
            form.set("catalog_name", next || undefined)
          }
          onSchemaChange={() => {}}
          requireCatalog
          requireSchema={false}
        />
        {catalogError && (
          <p className="text-xs text-destructive">{catalogError}</p>
        )}
      </div>
      <NameField form={form} name="name" placeholder="bronze" autoFocus />
      <CommentField form={form} />
      <StorageLocationPicker
        error={form.error("storage_root")}
        onChange={(location, { incomplete }) => {
          setStorageIncomplete(incomplete);
          form.set("storage_root", location);
        }}
      />
      <PropertiesField form={form} />
    </ResourceFormDialog>
  );
}

export function SchemaEditDialog({
  fullName,
  onClose,
}: {
  fullName: string;
  onClose: () => void;
}) {
  const update = useUpdateSchema();
  const followRename = useFollowRename("schema", fullName);
  const detail = useSchemaDetail(fullName);
  const form = useResourceForm<UpdateSchema>(
    updateSchema,
    {},
    { validate: requireNewName, patch: true },
  );

  const loaded = detail.data;
  const { seed } = form;
  useEffect(() => {
    if (loaded) {
      seed({
        new_name: loaded.name ?? leafName(fullName),
        comment: loaded.comment,
        properties: loaded.properties,
      });
    }
  }, [loaded, seed, fullName]);

  const submit = form.submit(() => {
    const body = form.changes(["new_name", "comment", "properties"]);
    if (Object.keys(body).length === 0) return onClose();
    if (body.new_name) body.new_name = body.new_name.trim();
    update.mutate(
      { params: { path: { full_name: fullName } }, body },
      {
        onSuccess: () => {
          toast.success(
            `Updated schema "${body.new_name ?? leafName(fullName)}"`,
          );
          followRename(body.new_name);
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="Edit schema"
      description={<span className="font-mono">{fullName}</span>}
      submitLabel="Save"
      pending={update.isPending}
      canSubmit={!!loaded}
      loadError={loadErrorOf(detail)}
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
      <PropertiesField
        form={form}
        disabled={!loaded}
        help="Saving replaces the whole property map."
      />
    </ResourceFormDialog>
  );
}
