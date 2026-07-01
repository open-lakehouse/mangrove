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
} from "@open-lakehouse/unity-catalog-client";
import type { RJSFSchema, UiSchema } from "@rjsf/utils";
import { Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { SchemaForm } from "../forms/SchemaForm";
import { cloneSchema, formSchemas } from "../forms/schemas";

import { CredentialDialog } from "./CredentialDialog";

const FORM_ID = "external-location-form";

interface ExternalLocationFormData {
  name?: string;
  new_name?: string;
  url?: string;
  credential_name?: string;
  comment?: string;
}

type LooseSchema = Record<string, unknown>;

/** Inject the available credential names as an enum so the field renders a select. */
function withCredentialEnum(base: RJSFSchema, names: string[]): RJSFSchema {
  const schema = cloneSchema(base);
  const props = (schema.properties ?? {}) as Record<string, LooseSchema>;
  if (props.credential_name && names.length > 0) {
    props.credential_name = { ...props.credential_name, enum: names };
  }
  return schema;
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
  const createLocation = useCreateExternalLocation();
  const updateLocation = useUpdateExternalLocation();
  const credentials = useCredentials();
  const existing = useExternalLocationDetail(name ?? "", {
    enabled: mode === "edit" && !!name,
  });

  const [formData, setFormData] = useState<ExternalLocationFormData>(() =>
    mode === "create" ? { name: "", url: "" } : {},
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
    () =>
      withCredentialEnum(
        mode === "create"
          ? formSchemas.createExternalLocation
          : formSchemas.updateExternalLocation,
        credentialNames,
      ),
    [mode, credentialNames],
  );

  const uiSchema: UiSchema = useMemo(() => {
    const base: UiSchema = {
      read_only: { "ui:widget": "hidden" },
      skip_validation: { "ui:widget": "hidden" },
      force: { "ui:widget": "hidden" },
      owner: { "ui:widget": "hidden" },
      url: { "ui:placeholder": "s3://bucket/path" },
      credential_name: { "ui:placeholder": "Select a storage credential" },
      comment: { "ui:placeholder": "Description (optional)" },
    };
    if (mode === "create") {
      return {
        ...base,
        "ui:order": ["name", "url", "credential_name", "comment", "*"],
        name: {
          "ui:placeholder": "my_external_location",
          "ui:autofocus": true,
        },
      };
    }
    return {
      ...base,
      "ui:order": ["new_name", "url", "credential_name", "comment", "*"],
      name: { "ui:widget": "hidden" },
      new_name: { "ui:title": "Name", "ui:placeholder": name },
    };
  }, [mode, name]);

  const loaded = existing.data;
  useEffect(() => {
    if (mode === "edit" && loaded) {
      setFormData({
        new_name: loaded.name,
        url: loaded.url,
        credential_name: loaded.credential_name,
        comment: loaded.comment,
      });
    }
  }, [mode, loaded]);

  const pending = createLocation.isPending || updateLocation.isPending;

  function submit(data: ExternalLocationFormData) {
    if (mode === "create") {
      createLocation.mutate(
        {
          body: {
            name: data.name ?? "",
            url: data.url ?? "",
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

    const renamed = data.new_name && data.new_name !== name;
    updateLocation.mutate(
      {
        params: { path: { name: name ?? "" } },
        body: {
          url: data.url || undefined,
          credential_name: data.credential_name || undefined,
          comment: data.comment || undefined,
          new_name: renamed ? data.new_name : undefined,
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
            <SchemaForm<ExternalLocationFormData>
              id={FORM_ID}
              schema={schema}
              uiSchema={uiSchema}
              formData={formData}
              disabled={pending || (mode === "edit" && existing.isLoading)}
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
            <Button type="submit" form={FORM_ID} size="sm" disabled={pending}>
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
