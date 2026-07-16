// Create / edit a Unity Catalog storage credential (AWS IAM role).
//
// The form is generated from the protobuf-derived JSON Schema; we tailor it at
// runtime to the AWS-only scope (hide the Azure/GCP credential configs and the
// purpose/advanced fields). After creation we surface the IAM trust details
// (`unity_catalog_iam_arn` + `external_id`) the user needs to finish wiring up
// the role, mirroring the Databricks Catalog Explorer flow.

import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@open-lakehouse/ui-kit";
import type { CredentialInfo } from "@open-lakehouse/unity-catalog-client";
import {
  parseUcError,
  useCreateCredential,
  useCredentialDetail,
  useUpdateCredential,
} from "@open-lakehouse/unity-catalog-client";
import type { RJSFSchema, UiSchema } from "@rjsf/utils";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { SchemaForm } from "../forms/SchemaForm";
import { cloneSchema, formSchemas } from "../forms/schemas";
import { awsIamRoleTrust } from "../types";

import { CopyField } from "./CopyField";

const FORM_ID = "credential-form";

const AZURE_GCP_FIELDS = [
  "azure_service_principal",
  "azure_managed_identity",
  "azure_storage_key",
  "databricks_gcp_service_account",
];

interface CredentialFormData {
  name?: string;
  purpose?: string;
  comment?: string;
  new_name?: string;
  aws_iam_role?: { role_arn?: string };
}

type LooseSchema = Record<string, unknown>;

/** Reduce the generated credential schema to the AWS IAM role scope. */
function tailorCredentialSchema(base: RJSFSchema): RJSFSchema {
  const schema = cloneSchema(base);
  const props = (schema.properties ?? {}) as Record<string, LooseSchema>;
  for (const field of AZURE_GCP_FIELDS) delete props[field];

  // Require the IAM role + its ARN so the form enforces what AWS credentials need.
  schema.required = Array.from(
    new Set([...(schema.required ?? []), "aws_iam_role"]),
  );
  const ref = (
    props.aws_iam_role as { $ref?: string } | undefined
  )?.$ref?.replace("#/$defs/", "");
  const defs = schema.$defs as Record<string, LooseSchema> | undefined;
  const awsDef = ref ? defs?.[ref] : undefined;
  if (awsDef) {
    awsDef.required = Array.from(
      new Set([...((awsDef.required as string[]) ?? []), "role_arn"]),
    );
  }
  return schema;
}

const AWS_ROLE_UI: UiSchema = {
  "ui:title": "AWS IAM role",
  role_arn: {
    "ui:title": "IAM role ARN",
    "ui:placeholder": "arn:aws:iam::<account-id>:role/<role-name>",
  },
  region: { "ui:widget": "hidden" },
  access_key_id: { "ui:widget": "hidden" },
  secret_access_key: { "ui:widget": "hidden" },
  session_token: { "ui:widget": "hidden" },
};

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
  const createCredential = useCreateCredential();
  const updateCredential = useUpdateCredential();
  const existing = useCredentialDetail(name ?? "", {
    enabled: mode === "edit" && !!name,
  });

  const [created, setCreated] = useState<CredentialInfo>();
  // `purpose` is required by the schema but fixed to STORAGE (and hidden), so it
  // must be seeded for the form to validate/submit.
  const [formData, setFormData] = useState<CredentialFormData>(() =>
    mode === "create" ? { name: "", purpose: "STORAGE" } : {},
  );

  const schema = useMemo(
    () =>
      tailorCredentialSchema(
        mode === "create"
          ? formSchemas.createCredential
          : formSchemas.updateCredential,
      ),
    [mode],
  );

  const uiSchema: UiSchema = useMemo(() => {
    const base: UiSchema = {
      purpose: { "ui:widget": "hidden" },
      read_only: { "ui:widget": "hidden" },
      skip_validation: { "ui:widget": "hidden" },
      force: { "ui:widget": "hidden" },
      owner: { "ui:widget": "hidden" },
      comment: { "ui:placeholder": "Description (optional)" },
      aws_iam_role: AWS_ROLE_UI,
    };
    if (mode === "create") {
      return {
        ...base,
        "ui:order": ["name", "aws_iam_role", "comment", "*"],
        name: { "ui:placeholder": "my_credential", "ui:autofocus": true },
      };
    }
    return {
      ...base,
      "ui:order": ["new_name", "aws_iam_role", "comment", "*"],
      // `name` identifies the credential (path param); rename uses `new_name`.
      name: { "ui:widget": "hidden" },
      new_name: { "ui:title": "Name", "ui:placeholder": name },
    };
  }, [mode, name]);

  // Seed the edit form once the current credential loads.
  const loaded = existing.data;
  useEffect(() => {
    if (mode === "edit" && loaded) {
      setFormData({
        // `name` is hidden but required by the update schema (path identifier).
        name: loaded.name,
        purpose: loaded.purpose,
        new_name: loaded.name,
        comment: loaded.comment,
        aws_iam_role: { role_arn: loaded.aws_iam_role?.role_arn },
      });
    }
  }, [mode, loaded]);

  const pending = createCredential.isPending || updateCredential.isPending;

  function submit(data: CredentialFormData) {
    const roleArn = data.aws_iam_role?.role_arn?.trim();
    if (!roleArn) {
      toast.error("An IAM role ARN is required.");
      return;
    }

    if (mode === "create") {
      createCredential.mutate(
        {
          body: {
            name: data.name ?? "",
            comment: data.comment || undefined,
            purpose: "STORAGE",
            aws_iam_role: { role_arn: roleArn },
          },
        },
        {
          onSuccess: (result) => {
            const createdName = result?.name ?? data.name ?? "";
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
          onError: (error) => toast.error(parseUcError(error)),
        },
      );
      return;
    }

    const renamed = data.new_name && data.new_name !== name;
    updateCredential.mutate(
      {
        params: { path: { name: name ?? "" } },
        body: {
          comment: data.comment || undefined,
          new_name: renamed ? data.new_name : undefined,
          aws_iam_role: { role_arn: roleArn },
        },
      },
      {
        onSuccess: () => {
          toast.success(`Updated credential "${name}"`);
          onClose();
        },
        onError: (error) => toast.error(parseUcError(error)),
      },
    );
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {created
              ? "Credential created"
              : mode === "create"
                ? "Create a storage credential"
                : "Edit credential"}
          </DialogTitle>
          {!created && (
            <DialogDescription>
              A storage credential holds the long-term cloud credential (an AWS
              IAM role) that grants access to cloud storage.
            </DialogDescription>
          )}
        </DialogHeader>

        {created ? (
          <div className="space-y-4 px-5 py-4">
            <p className="text-sm text-muted-foreground">
              Update your IAM role's trust policy with the following so Unity
              Catalog can assume it:
            </p>
            <CopyField
              label="Unity Catalog IAM ARN"
              value={awsIamRoleTrust(created).unity_catalog_iam_arn}
            />
            <CopyField
              label="External ID"
              value={awsIamRoleTrust(created).external_id}
            />
          </div>
        ) : (
          <div className="px-5 py-4">
            <SchemaForm<CredentialFormData>
              id={FORM_ID}
              schema={schema}
              uiSchema={uiSchema}
              formData={formData}
              disabled={pending || (mode === "edit" && existing.isLoading)}
              onChange={setFormData}
              onSubmit={submit}
            />
          </div>
        )}

        <DialogFooter>
          {created ? (
            <Button type="button" size="sm" onClick={onClose}>
              Done
            </Button>
          ) : (
            <>
              <Button type="button" variant="ghost" size="sm" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit" form={FORM_ID} size="sm" disabled={pending}>
                {pending ? "Saving…" : mode === "create" ? "Create" : "Save"}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
