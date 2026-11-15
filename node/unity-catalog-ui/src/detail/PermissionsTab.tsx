import {
  Badge,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@open-lakehouse/ui-kit";
import {
  displayPrivilege,
  type GrantAssignment,
  type GrantSecurableType,
  normalizePrivilege,
  OWNER_PRIVILEGE,
  parseUcError,
} from "@open-lakehouse/unity-catalog-client";
import {
  useGrants,
  useUpdateGrants,
} from "@open-lakehouse/unity-catalog-client/react";
import { Plus, X } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { SectionLabel } from "../SectionLabel.js";
import { DetailStates } from "./DetailStates.js";
import {
  GrantForm,
  type GrantFormSubmission,
  type GrantFormValue,
} from "./GrantForm.js";

/** What the grants API calls each securable type, for messages. */
const NOUN: Record<GrantSecurableType, string> = {
  metastore: "metastore",
  catalog: "catalog",
  schema: "schema",
  table: "table",
  function: "function",
  volume: "volume",
  registered_model: "model",
  external_location: "external location",
  credential: "credential",
};

function errorCode(error: unknown): string | undefined {
  if (!error || typeof error !== "object") return undefined;
  const body = error as { error_code?: string; errorCode?: string };
  return body.error_code ?? body.errorCode;
}

const isPermissionDenied = (error: unknown) =>
  errorCode(error) === "PERMISSION_DENIED";

/** The assignments with canonical privilege names, sorted by principal, owner
 *  first within each. */
function normalized(assignments: GrantAssignment[] | undefined) {
  return (assignments ?? [])
    .map((a) => ({
      principal: a.principal,
      privileges: (a.privileges ?? [])
        .map(normalizePrivilege)
        .sort((x, y) =>
          x === OWNER_PRIVILEGE
            ? -1
            : y === OWNER_PRIVILEGE
              ? 1
              : x.localeCompare(y),
        ),
    }))
    .filter((a) => a.privileges.length > 0)
    .sort((a, b) => a.principal.localeCompare(b.principal));
}

/**
 * The privileges granted directly on one securable, with controls to grant
 * and revoke them. Inherited privileges (granted on a parent) are not listed.
 * `OWNER` is shown but can't be revoked here: ownership moves through the
 * securable's owner field.
 */
export function PermissionsTab({
  securableType,
  fullName,
}: {
  securableType: GrantSecurableType;
  fullName: string;
}) {
  const grants = useGrants(securableType, fullName);
  const update = useUpdateGrants();
  const [granting, setGranting] = useState(false);
  const noun = NOUN[securableType];

  if (grants.error && isPermissionDenied(grants.error)) {
    return (
      <p className="text-sm text-muted-foreground">
        You don&apos;t have permission to view the grants on this {noun}.
      </p>
    );
  }
  if (!grants.data) {
    return <DetailStates isLoading={grants.isLoading} error={grants.error} />;
  }

  const assignments = normalized(grants.data.privilege_assignments);
  const path = { securable_type: securableType, full_name: fullName };

  function revoke(principal: string, privilege: string) {
    update.mutate(
      {
        params: { path },
        body: { changes: [{ principal, remove: [privilege] }] },
      },
      {
        onSuccess: () =>
          toast.success(
            `Revoked ${displayPrivilege(privilege)} from ${principal}`,
          ),
        onError: (error) => toast.error(parseUcError(error)),
      },
    );
  }

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <SectionLabel>Grants</SectionLabel>
        <Button size="sm" variant="outline" onClick={() => setGranting(true)}>
          <Plus className="h-3.5 w-3.5" />
          Grant
        </Button>
      </div>

      {assignments.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No privileges are granted directly on this {noun}.
        </p>
      ) : (
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b text-left text-xs text-muted-foreground">
              <th className="py-1.5 pr-4 font-medium">Principal</th>
              <th className="py-1.5 font-medium">Privileges</th>
            </tr>
          </thead>
          <tbody>
            {assignments.map(({ principal, privileges }) => (
              <tr key={principal} className="border-b last:border-0 align-top">
                <td className="py-2 pr-4 font-mono text-xs">{principal}</td>
                <td className="py-1.5">
                  <ul className="flex flex-wrap gap-1.5">
                    {privileges.map((privilege) => (
                      <li key={privilege}>
                        <PrivilegeBadge
                          privilege={privilege}
                          disabled={update.isPending}
                          onRevoke={
                            privilege === OWNER_PRIVILEGE
                              ? undefined
                              : () => revoke(principal, privilege)
                          }
                          principal={principal}
                        />
                      </li>
                    ))}
                  </ul>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <p className="text-xs text-muted-foreground">
        Privileges granted on a parent also apply here and are not listed.
      </p>

      {granting && (
        <GrantDialog
          securableType={securableType}
          fullName={fullName}
          onClose={() => setGranting(false)}
        />
      )}
    </section>
  );
}

function PrivilegeBadge({
  privilege,
  principal,
  onRevoke,
  disabled,
}: {
  privilege: string;
  principal: string;
  onRevoke?: () => void;
  disabled?: boolean;
}) {
  const label = displayPrivilege(privilege);
  if (!onRevoke) {
    return (
      <Badge
        variant="primary"
        title="Ownership changes through the owner field"
      >
        {label}
      </Badge>
    );
  }
  return (
    <Badge variant="outline" className="gap-1 pr-1">
      {label}
      <button
        type="button"
        disabled={disabled}
        aria-label={`Revoke ${label} from ${principal}`}
        className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
        onClick={onRevoke}
      >
        <X className="h-3 w-3" />
      </button>
    </Badge>
  );
}

function GrantDialog({
  securableType,
  fullName,
  onClose,
}: {
  securableType: GrantSecurableType;
  fullName: string;
  onClose: () => void;
}) {
  const update = useUpdateGrants();
  const [value, setValue] = useState<GrantFormValue>({
    principal: "",
    privileges: [],
  });
  const [error, setError] = useState<string | null>(null);

  function submit({ principal, add }: GrantFormSubmission) {
    setError(null);
    update.mutate(
      {
        params: {
          path: { securable_type: securableType, full_name: fullName },
        },
        body: { changes: [{ principal, add }] },
      },
      {
        onSuccess: () => {
          toast.success(
            `Granted ${add.map(displayPrivilege).join(", ")} to ${principal}`,
          );
          onClose();
        },
        onError: (err) => setError(parseUcError(err)),
      },
    );
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <GrantForm
          securableType={securableType}
          value={value}
          onChange={setValue}
          onSubmit={submit}
          pending={update.isPending}
          error={error}
          header={
            <DialogHeader>
              <DialogTitle>Grant privileges</DialogTitle>
              <DialogDescription>
                On {NOUN[securableType]}{" "}
                <span className="font-mono">{fullName}</span>
              </DialogDescription>
            </DialogHeader>
          }
          footer={({ canSubmit, pending }) => (
            <DialogFooter>
              <Button type="button" variant="ghost" onClick={onClose}>
                Cancel
              </Button>
              <Button type="submit" disabled={!canSubmit}>
                {pending ? "Granting…" : "Grant"}
              </Button>
            </DialogFooter>
          )}
        />
      </DialogContent>
    </Dialog>
  );
}
