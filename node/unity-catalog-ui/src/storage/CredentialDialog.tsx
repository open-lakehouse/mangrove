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
} from "@open-lakehouse/unity-catalog-client/react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
  CREDENTIAL_CREATE_SEED,
  type CredentialFormData,
  commentPatch,
  credentialEditSeed,
  credentialSchema,
  credentialUiSchema,
} from "../forms/dialog-forms.js";
import { SchemaForm } from "../forms/SchemaForm.js";
import { awsIamRoleTrust } from "../types.js";

import { CopyField } from "./CopyField.js";

const FORM_ID = "credential-form";

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
  const [formData, setFormData] = useState<CredentialFormData>(() =>
    mode === "create" ? CREDENTIAL_CREATE_SEED : {},
  );

  const schema = useMemo(() => credentialSchema(mode), [mode]);
  const uiSchema = useMemo(() => credentialUiSchema(mode, name), [mode, name]);

  // Seed the edit form once the current credential loads.
  const loaded = existing.data;
  useEffect(() => {
    if (mode === "edit" && loaded) setFormData(credentialEditSeed(loaded));
  }, [mode, loaded]);

  const pending = createCredential.isPending || updateCredential.isPending;

  function submit(data: CredentialFormData) {
    const roleArn = data.aws_iam_role?.role_arn?.trim();

    if (mode === "create") {
      if (!roleArn) {
        toast.error("An IAM role ARN is required.");
        return;
      }
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

    const newName = data.new_name?.trim();
    // Send the role only when it changed, so editing a credential of another
    // cloud never turns it into an AWS one.
    const roleChanged =
      !!roleArn && roleArn !== (loaded?.aws_iam_role?.role_arn ?? "");
    updateCredential.mutate(
      {
        params: { path: { name: name ?? "" } },
        body: {
          comment: commentPatch(data.comment, loaded?.comment),
          new_name: newName && newName !== name ? newName : undefined,
          aws_iam_role: roleChanged ? { role_arn: roleArn } : undefined,
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
            {mode === "edit" && existing.isError ? (
              <p role="alert" className="mb-3 text-xs text-destructive">
                {parseUcError(existing.error, "Could not load the credential.")}
              </p>
            ) : null}
            <SchemaForm<CredentialFormData>
              id={FORM_ID}
              schema={schema}
              uiSchema={uiSchema}
              formData={formData}
              disabled={pending || (mode === "edit" && !loaded)}
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
              <Button
                type="submit"
                form={FORM_ID}
                size="sm"
                disabled={pending || (mode === "edit" && !loaded)}
              >
                {pending ? "Saving…" : mode === "create" ? "Create" : "Save"}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
