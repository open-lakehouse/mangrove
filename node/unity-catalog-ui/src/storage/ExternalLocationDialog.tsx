// Create / edit a Unity Catalog external location: a storage path bound to a
// storage credential. The credential is chosen from the existing credentials
// (with an inline shortcut to create a new one), mirroring the Databricks
// Catalog Explorer flow where you pick the storage credential that authorizes
// the location.

import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@open-lakehouse/ui-kit";
import {
  parseUcError,
  useCreateExternalLocation,
  useCredentials,
  useExternalLocationDetail,
  useUpdateExternalLocation,
} from "@open-lakehouse/unity-catalog-client/react";
import { Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
  commentPatch,
  EXTERNAL_LOCATION_CREATE_SEED,
  type ExternalLocationFormData,
  externalLocationEditSeed,
  externalLocationSchema,
  externalLocationUiSchema,
} from "../forms/dialog-forms.js";
import { SchemaForm } from "../forms/SchemaForm.js";

import { CredentialDialog } from "./CredentialDialog.js";

const FORM_ID = "external-location-form";

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
  const createLocation = useCreateExternalLocation();
  const updateLocation = useUpdateExternalLocation();
  const credentials = useCredentials();
  const existing = useExternalLocationDetail(name ?? "", {
    enabled: mode === "edit" && !!name,
  });

  const [formData, setFormData] = useState<ExternalLocationFormData>(() =>
    mode === "create" ? EXTERNAL_LOCATION_CREATE_SEED : {},
  );
  const [showCredentialDialog, setShowCredentialDialog] = useState(false);

  const credentialNames = useMemo(
    () =>
      (credentials.data ?? [])
        .map((c) => c.name)
        .filter((n): n is string => !!n),
    [credentials.data],
  );

  const schema = useMemo(
    () => externalLocationSchema(mode, credentialNames),
    [mode, credentialNames],
  );
  const uiSchema = useMemo(
    () => externalLocationUiSchema(mode, name),
    [mode, name],
  );

  const loaded = existing.data;
  useEffect(() => {
    if (mode === "edit" && loaded)
      setFormData(externalLocationEditSeed(loaded));
  }, [mode, loaded]);

  const pending = createLocation.isPending || updateLocation.isPending;

  function submit(data: ExternalLocationFormData) {
    if (mode === "create") {
      createLocation.mutate(
        {
          body: {
            name: data.name?.trim() ?? "",
            url: data.url?.trim() ?? "",
            credential_name: data.credential_name ?? "",
            comment: data.comment || undefined,
          },
        },
        {
          onSuccess: () => {
            toast.success(`Created external location "${data.name}"`);
            onClose();
          },
          onError: (error) => toast.error(parseUcError(error)),
        },
      );
      return;
    }

    const newName = data.new_name?.trim();
    updateLocation.mutate(
      {
        params: { path: { name: name ?? "" } },
        body: {
          url: data.url && data.url !== loaded?.url ? data.url : undefined,
          credential_name:
            data.credential_name &&
            data.credential_name !== loaded?.credential_name
              ? data.credential_name
              : undefined,
          comment: commentPatch(data.comment, loaded?.comment),
          new_name: newName && newName !== name ? newName : undefined,
        },
      },
      {
        onSuccess: () => {
          toast.success(`Updated external location "${name}"`);
          onClose();
        },
        onError: (error) => toast.error(parseUcError(error)),
      },
    );
  }

  return (
    <>
      <Dialog open onOpenChange={(open) => !open && onClose()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {mode === "create"
                ? "Create an external location"
                : "Edit external location"}
            </DialogTitle>
            <DialogDescription>
              An external location combines a storage path with a storage
              credential that authorizes access to it.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 px-5 py-4">
            {mode === "edit" && existing.isError ? (
              <p role="alert" className="text-xs text-destructive">
                {parseUcError(
                  existing.error,
                  "Could not load the external location.",
                )}
              </p>
            ) : null}
            <SchemaForm<ExternalLocationFormData>
              id={FORM_ID}
              schema={schema}
              uiSchema={uiSchema}
              formData={formData}
              disabled={pending || (mode === "edit" && !loaded)}
              onChange={setFormData}
              onSubmit={submit}
            />
            <Button
              type="button"
              variant="link"
              size="sm"
              className="h-auto p-0 text-xs"
              onClick={() => setShowCredentialDialog(true)}
            >
              <Plus className="h-3.5 w-3.5" />
              New storage credential
            </Button>
          </div>

          <DialogFooter>
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
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {showCredentialDialog && (
        <CredentialDialog
          mode="create"
          onClose={() => setShowCredentialDialog(false)}
          onCreated={(createdName) => {
            credentials.refetch();
            setFormData((prev) => ({ ...prev, credential_name: createdName }));
          }}
        />
      )}
    </>
  );
}
