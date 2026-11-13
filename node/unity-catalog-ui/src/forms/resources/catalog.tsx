// Create and edit a catalog.

import type { components } from "@open-lakehouse/unity-catalog-client";
import {
  useCatalogDetail,
  useCreateCatalog,
  useUpdateCatalog,
} from "@open-lakehouse/unity-catalog-client/react";
import { ChevronRight } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { useRevealCreated } from "../../create-reconcile.js";
import { StorageLocationPicker } from "../../storage/StorageLocationPicker.js";
import {
  CommentField,
  NameField,
  PropertiesField,
  TextField,
} from "../fields.js";
import { ResourceFormDialog } from "../ResourceFormDialog.js";
import { createCatalog, updateCatalog } from "../request-fields.gen.js";
import type { FieldErrors } from "../spec.js";
import { useResourceForm } from "../useResourceForm.js";
import {
  createBody,
  loadErrorOf,
  requireNewName,
  STORAGE_INCOMPLETE,
  useFollowRename,
} from "./shared.js";

type CreateCatalog = components["schemas"]["CreateCatalog"];
type UpdateCatalog = components["schemas"]["UpdateCatalogRequest"];

export function CatalogCreateDialog({ onClose }: { onClose: () => void }) {
  const create = useCreateCatalog();
  const reveal = useRevealCreated();
  const [storageIncomplete, setStorageIncomplete] = useState(false);
  const [sharing, setSharing] = useState(false);

  const form = useResourceForm<CreateCatalog>(
    createCatalog,
    {},
    {
      validate: (values): FieldErrors => {
        const errors: FieldErrors = {};
        if (storageIncomplete) errors.storage_root = STORAGE_INCOMPLETE;
        // A Delta Sharing catalog needs both the provider and the share.
        if (!!values.provider_name !== !!values.share_name) {
          const missing = values.provider_name ? "share_name" : "provider_name";
          errors[missing] = "Required for a Delta Sharing catalog.";
        }
        return errors;
      },
    },
  );

  const submit = form.submit((values) => {
    const body = createBody<CreateCatalog>(values);
    create.mutate(
      { body },
      {
        onSuccess: () => {
          toast.success(`Created catalog "${body.name}"`);
          reveal({ kind: "catalog", name: body.name });
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="New catalog"
      description="A catalog is the top level of the three-level namespace and holds schemas."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <NameField form={form} name="name" placeholder="sales" autoFocus />
      <CommentField form={form} />
      <StorageLocationPicker
        error={form.error("storage_root")}
        onChange={(location, { incomplete }) => {
          setStorageIncomplete(incomplete);
          form.set("storage_root", location);
        }}
      />
      <PropertiesField form={form} />

      <div className="space-y-3">
        <button
          type="button"
          className="flex items-center gap-1 text-xs font-medium text-muted-foreground hover:text-foreground"
          aria-expanded={sharing}
          onClick={() => setSharing((open) => !open)}
        >
          <ChevronRight
            className={`h-3.5 w-3.5 transition-transform ${sharing ? "rotate-90" : ""}`}
          />
          Create from a Delta Share
        </button>
        {sharing && (
          <div className="space-y-3 rounded-md border bg-muted/30 p-3">
            <TextField
              form={form}
              name="provider_name"
              label="Provider"
              mono
              placeholder="partner_provider"
            />
            <TextField
              form={form}
              name="share_name"
              label="Share"
              mono
              placeholder="shared_sales"
            />
          </div>
        )}
      </div>
    </ResourceFormDialog>
  );
}

export function CatalogEditDialog({
  name,
  onClose,
}: {
  name: string;
  onClose: () => void;
}) {
  const update = useUpdateCatalog();
  const followRename = useFollowRename("catalog", name);
  const detail = useCatalogDetail(name);
  const form = useResourceForm<UpdateCatalog>(
    updateCatalog,
    {},
    { validate: requireNewName, patch: true },
  );

  const loaded = detail.data;
  const { seed } = form;
  useEffect(() => {
    if (loaded) {
      seed({
        new_name: loaded.name,
        comment: loaded.comment,
        properties: loaded.properties,
      });
    }
  }, [loaded, seed]);

  const submit = form.submit(() => {
    const body = form.changes(["new_name", "comment", "properties"]);
    if (Object.keys(body).length === 0) return onClose();
    if (body.new_name) body.new_name = body.new_name.trim();
    update.mutate(
      { params: { path: { name } }, body },
      {
        onSuccess: () => {
          toast.success(`Updated catalog "${body.new_name ?? name}"`);
          followRename(body.new_name);
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="Edit catalog"
      description={<span className="font-mono">{name}</span>}
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
