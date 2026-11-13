// Pieces the resource dialogs share: request-body cleanup, rename handling
// and the storage-location rule.

import { parseUcError } from "@open-lakehouse/unity-catalog-client/react";
import type { EditableKind } from "../../dialog-types.js";
import { renamedSelection, useCatalogSelection } from "../../selection.js";
import type { FieldErrors } from "../spec.js";

/**
 * A create body from form values: strings trimmed, empty strings dropped.
 * Validation already ran on the values, so required fields are present.
 */
export function createBody<T>(values: Partial<T>): T {
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(values)) {
    if (typeof value === "string") {
      const trimmed = value.trim();
      if (trimmed) out[key] = trimmed;
    } else if (value !== undefined) {
      out[key] = value;
    }
  }
  return out as T;
}

/** The last part of a dotted full name: `a.b.c` → `c`. */
export function leafName(fullName: string): string {
  return fullName.split(".").pop() ?? fullName;
}

/**
 * An update requires a name: `new_name` is optional in the API (omitted means
 * "keep"), but the form seeds it with the current name, so an empty field
 * means the user cleared it.
 */
export function requireNewName(values: { new_name?: unknown }): FieldErrors {
  return values.new_name ? {} : { new_name: "Required." };
}

/**
 * After a rename, keep the URL selection pointing at the renamed object, or
 * at the same object under a renamed catalog or schema.
 */
export function useFollowRename(kind: EditableKind, fullName: string) {
  const { selection, select } = useCatalogSelection();
  return (newName: string | undefined) => {
    if (!newName || !selection) return;
    const next = renamedSelection(selection, kind, fullName, newName);
    if (next) select(next);
  };
}

/** The error text for a failed detail load, if the load failed. */
export function loadErrorOf(query: {
  isError: boolean;
  error: unknown;
}): string | undefined {
  return query.isError
    ? parseUcError(query.error, "Could not load this object.")
    : undefined;
}

/** Message for a storage picker left without a location. */
export const STORAGE_INCOMPLETE =
  "Choose an external location or enter a storage URL, or turn managed storage back on.";
