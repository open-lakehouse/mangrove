// The Unity Catalog A2UI catalog: components an agent surface may place, on
// top of A2UI's basic catalog. Today one component, `UcGrantForm`, which
// renders the Permissions tab's `GrantForm` with its value bound to the
// surface's data model, so the agent prefills it and the user edits it.

import { createComponentImplementation } from "@a2ui/react/v0_9";
import { Catalog, CommonSchemas } from "@a2ui/web_core/v0_9";
import {
  displayPrivilege,
  GRANTABLE_PRIVILEGES,
  type GrantSecurableType,
  type PrincipalRef,
} from "@open-lakehouse/unity-catalog-client";
import { GrantForm } from "@open-lakehouse/unity-catalog-ui/grants";
import { z } from "zod";
import { A2UI_VERSION, type GrantStatus, UC_CATALOG_ID } from "./protocol.js";

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

function isSecurableType(value: unknown): value is GrantSecurableType {
  return typeof value === "string" && value in GRANTABLE_PRIVILEGES;
}

/** The grant form's API: what the server may set on a `UcGrantForm`. */
export const UcGrantFormApi = {
  name: "UcGrantForm",
  allowedParents: ["Surface", "Column", "Card"],
  schema: z.object({
    /** The grants API's securable type, e.g. `table`. */
    securableType: CommonSchemas.DynamicString,
    /** The securable's full name, e.g. `main.sales.orders`. */
    fullName: CommonSchemas.DynamicString,
    /** The principal to grant to (two-way bound). */
    principal: CommonSchemas.DynamicString,
    /** The privileges to grant, canonical spelling (two-way bound). */
    privileges: CommonSchemas.DynamicStringList,
    /** The current direct grants, `{principal, privileges}[]`, for context. */
    assignments: CommonSchemas.DynamicValue.optional(),
    /** Principals the agent's hint matched, `{kind, name, displayName?}[]`,
     *  offered when the hint was ambiguous. */
    candidates: CommonSchemas.DynamicValue.optional(),
    /** `{state, message?}`: idle, pending, ok or error. */
    status: CommonSchemas.DynamicValue.optional(),
    /** Fired on submit; its context carries the grant. */
    submit: CommonSchemas.Action,
  }),
};

interface Assignment {
  principal: string;
  privileges: string[];
}

function asAssignments(value: unknown): Assignment[] {
  if (!Array.isArray(value)) return [];
  return value.filter(
    (a): a is Assignment =>
      !!a &&
      typeof a.principal === "string" &&
      Array.isArray(a.privileges) &&
      a.privileges.length > 0,
  );
}

function asCandidates(value: unknown): PrincipalRef[] {
  if (!Array.isArray(value)) return [];
  return value.filter(
    (c): c is PrincipalRef => !!c && typeof c.name === "string" && !!c.name,
  );
}

function asStatus(value: unknown): GrantStatus {
  if (value && typeof value === "object" && "state" in value) {
    return value as GrantStatus;
  }
  return { state: "idle" };
}

export const UcGrantForm = createComponentImplementation(
  UcGrantFormApi,
  ({ props }) => {
    const securableType = props.securableType;
    const fullName = props.fullName ?? "";
    if (!isSecurableType(securableType)) {
      return (
        <p role="alert" className="text-sm text-destructive">
          Can&apos;t grant on securable type “{String(securableType)}”.
        </p>
      );
    }
    const status = asStatus(props.status);
    const assignments = asAssignments(props.assignments);
    const candidates = asCandidates(props.candidates);
    const principal = props.principal ?? "";
    const privileges = Array.isArray(props.privileges) ? props.privileges : [];
    const pending = status.state === "pending";
    // Privileges the agent suggested that don't apply here: the form can't
    // show them, so say so rather than drop them silently.
    const grantable = GRANTABLE_PRIVILEGES[securableType];
    const inapplicable = privileges.filter((p) => !grantable.includes(p));

    return (
      <div className="space-y-4">
        <GrantForm
          securableType={securableType}
          value={{ principal, privileges }}
          onChange={(value) => {
            if (value.principal !== principal)
              props.setPrincipal(value.principal);
            if (value.privileges !== privileges)
              props.setPrivileges(value.privileges);
          }}
          onSubmit={() => props.submit()}
          pending={pending}
          error={status.state === "error" ? status.message : null}
          header={
            <div className="space-y-2">
              <h2 className="text-base font-semibold">Grant privileges</h2>
              <p className="text-sm text-muted-foreground">
                On {NOUN[securableType]}{" "}
                <span className="font-mono">{fullName}</span>
              </p>
              {candidates.length > 1 && (
                <div className="space-y-1">
                  <p className="text-xs text-muted-foreground">
                    Several principals match. Pick one:
                  </p>
                  <ul className="flex flex-wrap gap-1.5">
                    {candidates.map((c) => (
                      <li key={`${c.kind}:${c.name}`}>
                        <button
                          type="button"
                          className="rounded border px-2 py-0.5 text-xs hover:bg-muted"
                          aria-pressed={c.name === principal}
                          disabled={pending}
                          onClick={() => props.setPrincipal(c.name)}
                        >
                          {c.displayName
                            ? `${c.displayName} (${c.name})`
                            : c.name}
                        </button>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          }
        />

        {inapplicable.length > 0 && (
          <p className="text-xs text-muted-foreground">
            Not grantable on a {NOUN[securableType]}:{" "}
            {inapplicable.map(displayPrivilege).join(", ")}
          </p>
        )}

        {status.state === "ok" && (
          <p role="status" className="text-sm text-success">
            {status.message ?? "Granted."}
          </p>
        )}

        {assignments.length > 0 && (
          <section aria-label="Current grants" className="space-y-1">
            <h3 className="text-xs font-medium text-muted-foreground">
              Granted directly on this {NOUN[securableType]}
            </h3>
            <ul className="space-y-0.5 text-xs">
              {assignments.map((a) => (
                <li key={a.principal}>
                  <span className="font-mono">{a.principal}</span>:{" "}
                  {a.privileges.map(displayPrivilege).join(", ")}
                </li>
              ))}
            </ul>
          </section>
        )}
      </div>
    );
  },
);

/** The Unity Catalog catalog, for a `MessageProcessor` next to the basic one. */
export const ucCatalog = new Catalog(UC_CATALOG_ID, A2UI_VERSION, [
  UcGrantForm,
]);
