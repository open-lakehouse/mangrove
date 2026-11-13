// Form state for a create/update request, driven by its generated field specs.
//
// The hook owns the body being edited, per-field validation from the specs
// (shown once a field is touched, and for every field after a submit attempt),
// extra cross-field rules from the caller, and the error the server returned.
// For updates, `changes()` builds a PATCH body with only the fields the user
// changed, so an unchanged field can't overwrite a concurrent edit.

import { parseUcError } from "@open-lakehouse/unity-catalog-client/react";
import { useCallback, useMemo, useRef, useState } from "react";
import { type FieldErrors, type FieldSpec, validateRequest } from "./spec.js";

export type Specs = Record<string, FieldSpec>;

type Values = Record<string, unknown>;

export interface ResourceForm<T extends Values> {
  specs: Specs;
  values: Partial<T>;
  /** Set one field. Clearing a text field stores `undefined`. */
  set: <K extends keyof T & string>(name: K, value: T[K] | undefined) => void;
  /** Mark a field touched (on blur) so its error shows. */
  touch: (name: string) => void;
  /** The error to show for a field, if any. */
  error: (name: string) => string | undefined;
  /** All current errors, shown or not. */
  errors: FieldErrors;
  valid: boolean;
  /** The server's error for the last submit, when no field claimed it. */
  formError: string | undefined;
  /** Whether a field differs from its initial value. */
  dirty: (name: string) => boolean;
  /**
   * The changed fields among `names`, as a PATCH body. A cleared text field is
   * sent as `""` and a cleared map as `{}`: the API reads an omitted field as
   * "unchanged".
   */
  changes: <K extends keyof T & string>(names: readonly K[]) => Partial<T>;
  /** Validate everything; call `onValid` with the values if they pass. */
  submit: (
    onValid: (values: Partial<T>) => void,
  ) => (e?: { preventDefault?: () => void }) => void;
  /** Route a request failure to the field it names, or to the form. */
  fail: (error: unknown) => void;
  /** Replace all values and the baseline `dirty` compares against. */
  reset: (values: Partial<T>) => void;
  /**
   * `reset` the first time only: seed an edit form from the loaded object
   * without a later refetch overwriting what the user typed.
   */
  seed: (values: Partial<T>) => void;
}

export interface ResourceFormOptions<T extends Values> {
  /** Cross-field rules and requirements beyond the specs. */
  validate?: (values: Partial<T>) => FieldErrors;
  /**
   * An update form: validate only the fields that changed. A PATCH leaves the
   * others as they are, and some can't be valid as loaded (secrets the server
   * never returns).
   */
  patch?: boolean;
}

/** Unset, empty text and an empty map all mean "no value". */
function blank(value: unknown): boolean {
  if (value === undefined || value === null || value === "") return true;
  return (
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.keys(value).length === 0
  );
}

function equal(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (blank(a) && blank(b)) return true;
  if (typeof a === "object" && typeof b === "object" && a && b) {
    return JSON.stringify(a) === JSON.stringify(b);
  }
  return false;
}

/**
 * The value to PATCH for a field. A cleared field has to be sent explicitly:
 * `""` for text, `{}` for a map. An omitted field means "unchanged".
 */
function clearedValue(spec: FieldSpec | undefined, value: unknown): unknown {
  if (value !== undefined && value !== "") return value;
  if (spec?.kind === "string") return "";
  if (spec?.kind === "map") return {};
  return value;
}

/** The field a server error message names, if it names exactly one we know. */
function fieldInMessage(message: string, names: string[]): string | undefined {
  const hits = names.filter((name) =>
    new RegExp(`\\b${name}\\b`).test(message),
  );
  return hits.length === 1 ? hits[0] : undefined;
}

export function useResourceForm<T extends Values>(
  specs: Specs,
  initial: Partial<T>,
  options: ResourceFormOptions<T> = {},
): ResourceForm<T> {
  const [values, setValues] = useState<Partial<T>>(initial);
  const baseline = useRef<Partial<T>>(initial);
  const [touched, setTouched] = useState<Set<string>>(() => new Set());
  const [submitted, setSubmitted] = useState(false);
  const [serverErrors, setServerErrors] = useState<FieldErrors>({});
  const [formError, setFormError] = useState<string>();
  const { validate, patch = false } = options;

  const check = useCallback(
    (current: Partial<T>): FieldErrors => {
      const checked = patch
        ? Object.fromEntries(
            Object.entries(specs).filter(
              ([name]) =>
                !equal(
                  (current as Values)[name],
                  (baseline.current as Values)[name],
                ),
            ),
          )
        : specs;
      return {
        ...validateRequest(checked, current as Values),
        ...(validate?.(current) ?? {}),
      };
    },
    [specs, validate, patch],
  );

  const errors = useMemo<FieldErrors>(
    () => ({ ...check(values), ...serverErrors }),
    [check, values, serverErrors],
  );

  const set = useCallback(
    <K extends keyof T & string>(name: K, value: T[K] | undefined) => {
      setValues((prev) => ({ ...prev, [name]: value }));
      // A server error is about the submitted value; editing clears it.
      setServerErrors((prev) => {
        if (!(name in prev)) return prev;
        const { [name]: _, ...rest } = prev;
        return rest;
      });
      setFormError(undefined);
    },
    [],
  );

  const touch = useCallback((name: string) => {
    setTouched((prev) => (prev.has(name) ? prev : new Set(prev).add(name)));
  }, []);

  const error = (name: string) =>
    submitted || touched.has(name) || name in serverErrors
      ? errors[name]
      : undefined;

  const dirty = (name: string) =>
    !equal((values as Values)[name], (baseline.current as Values)[name]);

  function changes<K extends keyof T & string>(names: readonly K[]) {
    const out: Partial<T> = {};
    for (const name of names) {
      if (!dirty(name)) continue;
      out[name] = clearedValue(specs[name], values[name]) as T[K];
    }
    return out;
  }

  function submit(onValid: (values: Partial<T>) => void) {
    return (e?: { preventDefault?: () => void }) => {
      e?.preventDefault?.();
      setSubmitted(true);
      setFormError(undefined);
      if (Object.keys(check(values)).length === 0) onValid(values);
    };
  }

  function fail(err: unknown) {
    const message = parseUcError(err);
    const field = fieldInMessage(message, Object.keys(specs));
    if (field) {
      setServerErrors({ [field]: message });
    } else {
      setFormError(message);
    }
  }

  // Stable, so an edit form can reset from an effect when its data loads.
  const reset = useCallback((next: Partial<T>) => {
    baseline.current = next;
    setValues(next);
    setTouched(new Set());
    setSubmitted(false);
    setServerErrors({});
    setFormError(undefined);
  }, []);

  const seeded = useRef(false);
  const seed = useCallback(
    (next: Partial<T>) => {
      if (seeded.current) return;
      seeded.current = true;
      reset(next);
    },
    [reset],
  );

  return {
    specs,
    values,
    set,
    touch,
    error,
    errors,
    valid: Object.keys(errors).length === 0,
    formError,
    dirty,
    changes,
    submit,
    fail,
    reset,
    seed,
  };
}
