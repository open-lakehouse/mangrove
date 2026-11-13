// Create / edit a Unity Catalog external location: a storage path bound to a
// credential. The credential is chosen from the existing credentials (with an
// inline shortcut to create a new one), mirroring the Databricks Catalog
// Explorer flow where you pick the credential that authorizes the location.

import {
  Button,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@open-lakehouse/ui-kit";
import type { components } from "@open-lakehouse/unity-catalog-client";
import {
  useCreateExternalLocation,
  useCredentials,
  useExternalLocationDetail,
  useUpdateExternalLocation,
} from "@open-lakehouse/unity-catalog-client/react";
import { Plus } from "lucide-react";
import { useEffect, useId, useMemo, useState } from "react";
import { toast } from "sonner";
import {
  CommentField,
  FieldShell,
  NameField,
  SwitchField,
  TextField,
} from "../forms/fields.js";
import { ResourceFormDialog } from "../forms/ResourceFormDialog.js";
import {
  createExternalLocation,
  updateExternalLocation,
} from "../forms/request-fields.gen.js";
import {
  createBody,
  loadErrorOf,
  requireNewName,
} from "../forms/resources/shared.js";
import {
  type ResourceForm,
  useResourceForm,
} from "../forms/useResourceForm.js";
import { CredentialDialog } from "./CredentialDialog.js";

type CreateLocation = components["schemas"]["CreateExternalLocation"];
type UpdateLocation = components["schemas"]["UpdateExternalLocationRequest"];

/** Pick the credential that authorizes the location, or create one inline. */
function CredentialPicker({
  form,
  disabled,
}: {
  form: ResourceForm<{ credential_name?: string }>;
  disabled?: boolean;
}) {
  const id = useId();
  const credentials = useCredentials();
  const [creating, setCreating] = useState(false);
  const value = form.values.credential_name;
  // Always offer the current value, even before the list loads or when it is
  // on a later page, so the select never drops it.
  const names = useMemo(() => {
    const listed = (credentials.data ?? [])
      .map((c) => c.name)
      .filter((n): n is string => !!n);
    return value && !listed.includes(value) ? [value, ...listed] : listed;
  }, [credentials.data, value]);

  return (
    <>
      <FieldShell
        id={id}
        label="Credential"
        required={form.specs.credential_name?.required}
        help="The credential that authorizes access to this path."
        error={form.error("credential_name")}
      >
        <Select
          value={value || undefined}
          onValueChange={(next) => {
            // Radix reports "" when the value isn't among the options; that
            // is not a user choice and must not clear the credential.
            if (!next) return;
            form.set("credential_name", next);
            form.touch("credential_name");
          }}
          disabled={disabled}
        >
          <SelectTrigger id={id} aria-invalid={!!form.error("credential_name")}>
            <SelectValue
              placeholder={
                credentials.isLoading
                  ? "Loading credentials…"
                  : names.length
                    ? "Select a credential"
                    : "No credentials yet"
              }
            />
          </SelectTrigger>
          <SelectContent>
            {names.map((n) => (
              <SelectItem key={n} value={n}>
                {n}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          type="button"
          variant="link"
          size="sm"
          className="h-auto p-0 text-xs"
          disabled={disabled}
          onClick={() => setCreating(true)}
        >
          <Plus className="h-3.5 w-3.5" />
          New credential
        </Button>
      </FieldShell>

      {creating && (
        <CredentialDialog
          mode="create"
          onClose={() => setCreating(false)}
          onCreated={(createdName) => {
            credentials.refetch();
            form.set("credential_name", createdName);
          }}
        />
      )}
    </>
  );
}

export function ExternalLocationDialog({
  mode,
  name,
  onClose,
}: {
  mode: "create" | "edit";
  /** Required in edit mode: the external location being edited. */
  name?: string;
  onClose: () => void;
}) {
  return mode === "create" ? (
    <ExternalLocationCreate onClose={onClose} />
  ) : (
    <ExternalLocationEdit name={name ?? ""} onClose={onClose} />
  );
}

function ExternalLocationCreate({ onClose }: { onClose: () => void }) {
  const create = useCreateExternalLocation();
  const form = useResourceForm<CreateLocation>(createExternalLocation, {});

  const submit = form.submit((values) => {
    const body = createBody<CreateLocation>(values);
    create.mutate(
      { body },
      {
        onSuccess: () => {
          toast.success(`Created external location "${body.name}"`);
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="New external location"
      description="An external location combines a storage path with a credential that authorizes access to it."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <NameField form={form} name="name" placeholder="landing_zone" autoFocus />
      <TextField
        form={form}
        name="url"
        label="URL"
        mono
        placeholder="s3://bucket/path"
        help="The storage path this location covers, including everything below it."
      />
      <CredentialPicker form={form} />
      <CommentField form={form} />
      <SwitchField form={form} name="read_only" label="Read only" />
      <SwitchField form={form} name="skip_validation" label="Skip validation" />
    </ResourceFormDialog>
  );
}

function ExternalLocationEdit({
  name,
  onClose,
}: {
  name: string;
  onClose: () => void;
}) {
  const update = useUpdateExternalLocation();
  const detail = useExternalLocationDetail(name);
  const form = useResourceForm<UpdateLocation>(
    updateExternalLocation,
    {},
    { validate: requireNewName, patch: true },
  );

  const loaded = detail.data;
  const { seed } = form;
  useEffect(() => {
    if (loaded) {
      seed({
        new_name: loaded.name,
        url: loaded.url,
        credential_name: loaded.credential_name,
        comment: loaded.comment,
        read_only: loaded.read_only,
      });
    }
  }, [loaded, seed]);

  const submit = form.submit(() => {
    const body = form.changes([
      "new_name",
      "url",
      "credential_name",
      "comment",
      "read_only",
    ]);
    if (Object.keys(body).length === 0) return onClose();
    if (body.new_name) body.new_name = body.new_name.trim();
    if (body.url) body.url = body.url.trim();
    update.mutate(
      { params: { path: { name } }, body },
      {
        onSuccess: () => {
          toast.success(`Updated external location "${body.new_name ?? name}"`);
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="Edit external location"
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
      <TextField
        form={form}
        name="url"
        label="URL"
        mono
        disabled={!loaded}
        help={null}
      />
      <CredentialPicker form={form} disabled={!loaded} />
      <CommentField form={form} disabled={!loaded} />
      <SwitchField
        form={form}
        name="read_only"
        label="Read only"
        disabled={!loaded}
      />
    </ResourceFormDialog>
  );
}
