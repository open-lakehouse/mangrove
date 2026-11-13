// Create / edit a Unity Catalog credential.
//
// The fields come from the generated request specs (see
// forms/request-fields.gen.ts). A credential holds exactly one cloud
// configuration, so the form picks the type first and shows only that type's
// fields; on edit the type is fixed. After creating an AWS credential we show
// the IAM trust details (`unity_catalog_iam_arn` + `external_id`) the user
// needs to finish wiring up the role, as Databricks Catalog Explorer does.

import {
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@open-lakehouse/ui-kit";
import type {
  CredentialInfo,
  components,
} from "@open-lakehouse/unity-catalog-client";
import {
  useCreateCredential,
  useCredentialDetail,
  useUpdateCredential,
} from "@open-lakehouse/unity-catalog-client/react";
import { useEffect, useId, useState } from "react";
import { toast } from "sonner";
import {
  CommentField,
  EnumField,
  NameField,
  ObjectFields,
  SwitchField,
} from "../forms/fields.js";
import { ResourceFormDialog } from "../forms/ResourceFormDialog.js";
import {
  createCredential,
  updateCredential,
} from "../forms/request-fields.gen.js";
import {
  createBody,
  loadErrorOf,
  requireNewName,
} from "../forms/resources/shared.js";
import { useResourceForm } from "../forms/useResourceForm.js";
import { awsIamRoleTrust } from "../types.js";
import { CopyField } from "./CopyField.js";

type CreateCredential = components["schemas"]["CreateCredentialRequest"];
type UpdateCredential = components["schemas"]["UpdateCredentialRequest"];

/** The mutually exclusive cloud configurations a credential can hold. */
export const CREDENTIAL_TYPES = [
  "aws_iam_role",
  "azure_managed_identity",
  "azure_service_principal",
  "azure_storage_key",
  "databricks_gcp_service_account",
] as const;

export type CredentialType = (typeof CREDENTIAL_TYPES)[number];

const TYPE_LABELS: Record<CredentialType, string> = {
  aws_iam_role: "AWS IAM role",
  azure_managed_identity: "Azure managed identity",
  azure_service_principal: "Azure service principal",
  azure_storage_key: "Azure storage account key",
  databricks_gcp_service_account: "GCP service account (Databricks-managed)",
};

/** Per-type presentation: which fields are advanced, masked, how labelled. */
const TYPE_FIELDS: Record<
  CredentialType,
  {
    labels?: Record<string, string>;
    placeholders?: Record<string, string>;
    advanced?: string[];
    secret?: string[];
  }
> = {
  aws_iam_role: {
    labels: { role_arn: "IAM role ARN" },
    placeholders: { role_arn: "arn:aws:iam::<account-id>:role/<role-name>" },
    advanced: ["region", "access_key_id", "secret_access_key", "session_token"],
    secret: ["secret_access_key", "session_token"],
  },
  azure_managed_identity: {
    placeholders: {
      access_connector_id:
        "/subscriptions/<guid>/resourceGroups/<rg>/providers/Microsoft.Databricks/accessConnectors/<name>",
    },
  },
  azure_service_principal: {
    advanced: ["federated_token_file"],
    secret: ["client_secret"],
  },
  azure_storage_key: { secret: ["account_key"] },
  databricks_gcp_service_account: {},
};

/** The configuration type a stored credential holds. */
export function credentialType(
  credential: Partial<Record<CredentialType, unknown>>,
): CredentialType | undefined {
  return CREDENTIAL_TYPES.find((type) => credential[type] != null);
}

const PURPOSE_LABELS: Record<string, string> = {
  STORAGE: "Storage: access cloud storage",
  SERVICE: "Service: call cloud services",
};

/** The editable part of a stored type configuration (no output-only fields). */
function editableConfig(
  type: CredentialType,
  credential: CredentialInfo,
): Record<string, unknown> {
  const spec = updateCredential[type];
  const stored = (credential[type] ?? {}) as Record<string, unknown>;
  const out: Record<string, unknown> = {};
  for (const key of Object.keys(spec.fields)) {
    if (stored[key] !== undefined) out[key] = stored[key];
  }
  return out;
}

function TypePicker({
  value,
  onChange,
}: {
  value: CredentialType;
  onChange: (type: CredentialType) => void;
}) {
  const id = useId();
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>Credential type</Label>
      <Select
        value={value}
        onValueChange={(next) => onChange(next as CredentialType)}
      >
        <SelectTrigger id={id}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {CREDENTIAL_TYPES.map((type) => (
            <SelectItem key={type} value={type}>
              {TYPE_LABELS[type]}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

export function CredentialDialog({
  mode,
  name,
  onClose,
  onCreated,
}: {
  mode: "create" | "edit";
  /** Required in edit mode: the credential being edited. */
  name?: string;
  onClose: () => void;
  /** Fired after a successful create with the new credential's name. */
  onCreated?: (name: string) => void;
}) {
  return mode === "create" ? (
    <CredentialCreate onClose={onClose} onCreated={onCreated} />
  ) : (
    <CredentialEdit name={name ?? ""} onClose={onClose} />
  );
}

function CredentialCreate({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated?: (name: string) => void;
}) {
  const create = useCreateCredential();
  const [type, setType] = useState<CredentialType>("aws_iam_role");
  const [created, setCreated] = useState<CredentialInfo>();
  const form = useResourceForm<CreateCredential>(createCredential, {
    purpose: "STORAGE",
    aws_iam_role: {} as CreateCredential["aws_iam_role"],
  });

  function chooseType(next: CredentialType) {
    // Only the chosen configuration is sent; start it empty so its required
    // fields validate.
    for (const other of CREDENTIAL_TYPES) form.set(other, undefined);
    form.set(next, {} as never);
    setType(next);
  }

  const submit = form.submit((values) => {
    const body = createBody<CreateCredential>(values);
    for (const other of CREDENTIAL_TYPES) {
      if (other !== type) delete body[other];
    }
    create.mutate(
      { body },
      {
        onSuccess: (result) => {
          const createdName = result?.name ?? body.name;
          toast.success(`Created credential "${createdName}"`);
          onCreated?.(createdName);
          // Show trust details if the server returned them; otherwise close.
          const trust = awsIamRoleTrust(result);
          if (trust.external_id || trust.unity_catalog_iam_arn) {
            setCreated(result);
          } else {
            onClose();
          }
        },
        onError: form.fail,
      },
    );
  });

  if (created) {
    const trust = awsIamRoleTrust(created);
    return (
      <ResourceFormDialog
        title="Credential created"
        submitLabel="Done"
        onSubmit={onClose}
        onClose={onClose}
        footer={
          <Button type="button" size="sm" onClick={onClose}>
            Done
          </Button>
        }
      >
        <p className="text-sm text-muted-foreground">
          Update your IAM role's trust policy with the following so Unity
          Catalog can assume it:
        </p>
        <CopyField
          label="Unity Catalog IAM ARN"
          value={trust.unity_catalog_iam_arn}
        />
        <CopyField label="External ID" value={trust.external_id} />
      </ResourceFormDialog>
    );
  }

  return (
    <ResourceFormDialog
      title="New credential"
      description="A credential holds the long-term cloud identity that grants access to storage or services."
      submitLabel="Create"
      pendingLabel="Creating…"
      pending={create.isPending}
      formError={form.formError}
      onSubmit={submit}
      onClose={onClose}
    >
      <NameField
        form={form}
        name="name"
        placeholder="my_credential"
        autoFocus
      />
      <EnumField
        form={form}
        name="purpose"
        help={null}
        labels={PURPOSE_LABELS}
      />
      <TypePicker value={type} onChange={chooseType} />
      <div className="rounded-md border bg-muted/30 p-3">
        <ObjectFields form={form} name={type} {...TYPE_FIELDS[type]} />
      </div>
      <CommentField form={form} />
      <SwitchField form={form} name="read_only" label="Read only" />
      <SwitchField form={form} name="skip_validation" label="Skip validation" />
    </ResourceFormDialog>
  );
}

function CredentialEdit({
  name,
  onClose,
}: {
  name: string;
  onClose: () => void;
}) {
  const update = useUpdateCredential();
  const detail = useCredentialDetail(name);
  const form = useResourceForm<UpdateCredential>(
    updateCredential,
    {},
    { validate: requireNewName, patch: true },
  );

  const loaded = detail.data;
  const type = loaded ? credentialType(loaded) : undefined;
  const { seed } = form;
  useEffect(() => {
    if (!loaded) return;
    const current = credentialType(loaded);
    seed({
      new_name: loaded.name,
      comment: loaded.comment,
      read_only: loaded.read_only,
      ...(current ? { [current]: editableConfig(current, loaded) } : {}),
    });
  }, [loaded, seed]);

  const submit = form.submit(() => {
    const body = form.changes([
      "new_name",
      "comment",
      "read_only",
      ...(type ? [type] : []),
    ]);
    if (Object.keys(body).length === 0) return onClose();
    if (body.new_name) body.new_name = body.new_name.trim();
    update.mutate(
      { params: { path: { name } }, body },
      {
        onSuccess: () => {
          toast.success(`Updated credential "${body.new_name ?? name}"`);
          onClose();
        },
        onError: form.fail,
      },
    );
  });

  return (
    <ResourceFormDialog
      title="Edit credential"
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
      {type && (
        <div className="space-y-2 rounded-md border bg-muted/30 p-3">
          <p className="text-xs font-medium">{TYPE_LABELS[type]}</p>
          <ObjectFields form={form} name={type} {...TYPE_FIELDS[type]} />
        </div>
      )}
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
