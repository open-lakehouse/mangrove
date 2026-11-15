import { Button, Label } from "@open-lakehouse/ui-kit";
import {
  displayPrivilege,
  GRANTABLE_PRIVILEGES,
  type GrantSecurableType,
} from "@open-lakehouse/unity-catalog-client";
import { type FormEvent, type ReactNode, useId } from "react";

import { PrincipalInput } from "./PrincipalInput.js";

/** What the grant form edits: one principal and the privileges to add. */
export interface GrantFormValue {
  principal: string;
  privileges: string[];
}

/** A grant the form submits: a trimmed principal and the privileges to add,
 *  in canonical spelling and the menu's order. */
export interface GrantFormSubmission {
  principal: string;
  add: string[];
}

/** The privileges in `value` that apply to `securableType`, in menu order. */
export function grantSubmission(
  securableType: GrantSecurableType,
  value: GrantFormValue,
): GrantFormSubmission {
  const add = GRANTABLE_PRIVILEGES[securableType].filter((p) =>
    value.privileges.includes(p),
  );
  return { principal: value.principal.trim(), add };
}

/** Whether `value` names a principal and at least one applicable privilege. */
export function canSubmitGrant(
  securableType: GrantSecurableType,
  value: GrantFormValue,
): boolean {
  const { principal, add } = grantSubmission(securableType, value);
  return principal.length > 0 && add.length > 0;
}

/**
 * The fields of a grant: a principal (searching the identity directory in
 * scope, if any) and the privileges that apply to `securableType`. Controlled
 * and side-effect free; the caller sends the grant from `onSubmit`, which only
 * fires for a submittable value. Hosts wrap it: the Permissions tab in a
 * dialog, the agent surface (`@open-lakehouse/uc-a2ui`) in an A2UI component.
 */
export function GrantForm({
  securableType,
  value,
  onChange,
  onSubmit,
  pending = false,
  error,
  header,
  footer,
}: {
  securableType: GrantSecurableType;
  value: GrantFormValue;
  onChange: (value: GrantFormValue) => void;
  onSubmit: (grant: GrantFormSubmission) => void;
  /** A submission is in flight: inputs are disabled. */
  pending?: boolean;
  /** Why the last submission failed, shown above the footer. */
  error?: string | null;
  /** Rendered first, inside the form. */
  header?: ReactNode;
  /** Replaces the default footer (one submit button). Render a
   *  `type="submit"` button in it; `canSubmit` says whether to enable it. */
  footer?: (state: { canSubmit: boolean; pending: boolean }) => ReactNode;
}) {
  const principalId = useId();
  const privileges = GRANTABLE_PRIVILEGES[securableType];
  const canSubmit = !pending && canSubmitGrant(securableType, value);

  function toggle(privilege: string, on: boolean) {
    const current = value.privileges.filter((p) => p !== privilege);
    onChange({
      ...value,
      privileges: on ? [...current, privilege] : current,
    });
  }

  function submit(e: FormEvent) {
    e.preventDefault();
    if (!canSubmit) return;
    onSubmit(grantSubmission(securableType, value));
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      {header}

      <div className="space-y-1.5">
        <Label htmlFor={principalId}>Principal</Label>
        <PrincipalInput
          id={principalId}
          value={value.principal}
          onChange={(principal) => onChange({ ...value, principal })}
          disabled={pending}
        />
      </div>

      <fieldset className="space-y-1.5">
        <legend className="mb-1.5 text-sm font-medium">Privileges</legend>
        <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
          {privileges.map((privilege) => (
            <label key={privilege} className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                className="accent-primary"
                checked={value.privileges.includes(privilege)}
                disabled={pending}
                onChange={(e) => toggle(privilege, e.target.checked)}
              />
              {displayPrivilege(privilege)}
            </label>
          ))}
        </div>
      </fieldset>

      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}

      {footer ? (
        footer({ canSubmit, pending })
      ) : (
        <div className="flex justify-end">
          <Button type="submit" disabled={!canSubmit}>
            {pending ? "Granting…" : "Grant"}
          </Button>
        </div>
      )}
    </form>
  );
}
