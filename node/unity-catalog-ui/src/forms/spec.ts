// Field specs: what a request body field accepts, as declared in the OpenAPI
// spec (openapi/openapi.yaml, generated from the protos).
//
// `request-fields.gen.ts` holds one spec map per create/update request. Fields
// the protos mark OUTPUT_ONLY (`readOnly` in OpenAPI) are never emitted, so a
// form built from these specs can't send them. Each map is checked against the
// openapi-typescript body type with `satisfies FieldSpecs<Body>`, so the specs
// fail to compile when the spec and the generated client types drift.

interface BaseSpec {
  /** The API requires the field (`google.api.field_behavior = REQUIRED`). */
  required?: boolean;
  title?: string;
  description?: string;
}

export interface StringSpec extends BaseSpec {
  kind: "string";
  minLength?: number;
  maxLength?: number;
  pattern?: string;
}

export interface EnumSpec<V extends string = string> extends BaseSpec {
  kind: "enum";
  values: readonly V[];
}

export interface BooleanSpec extends BaseSpec {
  kind: "boolean";
}

export interface NumberSpec extends BaseSpec {
  kind: "number";
  minimum?: number;
  maximum?: number;
}

/** A `map<string, string>` field, e.g. `properties`. */
export interface MapSpec extends BaseSpec {
  kind: "map";
}

export interface ObjectSpec<T = unknown> extends BaseSpec {
  kind: "object";
  fields: FieldSpecs<T>;
}

export type FieldSpec =
  | StringSpec
  | EnumSpec
  | BooleanSpec
  | NumberSpec
  | MapSpec
  | ObjectSpec;

// openapi-typescript marks `readOnly` properties `readonly`. Those are output
// only, so they have no spec.
type IfEquals<X, Y, A, B> =
  (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y ? 1 : 2 ? A : B;

type WritableKeys<T> = {
  [P in keyof T]-?: IfEquals<
    { [Q in P]: T[P] },
    { -readonly [Q in P]: T[P] },
    P,
    never
  >;
}[keyof T];

/** The spec a field of TypeScript type `V` must have. */
type SpecFor<V> = [V] extends [string]
  ? string extends V
    ? StringSpec
    : EnumSpec<V>
  : [V] extends [boolean]
    ? BooleanSpec
    : [V] extends [number]
      ? NumberSpec
      : [V] extends [Record<string, string>]
        ? string extends keyof V
          ? MapSpec
          : ObjectSpec<V>
        : V extends object
          ? ObjectSpec<V>
          : never;

/** One spec per writable field of the request body `T`, no more, no less. */
export type FieldSpecs<T> = {
  [K in WritableKeys<T>]-?: SpecFor<NonNullable<T[K]>>;
};

// ── Validation ──────────────────────────────────────────────────────────────

function isEmpty(value: unknown): boolean {
  return value === undefined || value === null || value === "";
}

// Patterns are compiled without the `u` flag: the protovalidate URI pattern
// uses identity escapes (`\-` in a class) that unicode mode rejects.
const patterns = new Map<string, RegExp>();
function compile(pattern: string): RegExp {
  let re = patterns.get(pattern);
  if (!re) {
    re = new RegExp(pattern);
    patterns.set(pattern, re);
  }
  return re;
}

/** Messages for the identifier and URL patterns the specs use. */
function patternMessage(pattern: string): string {
  if (pattern === "^[a-z][0-9a-z_]*[0-9a-z]$") {
    return "Use lowercase letters, digits and underscores; start with a letter and don't end with an underscore.";
  }
  if (pattern === "^[a-z][0-9a-z-_]*[0-9a-z]$") {
    return "Use lowercase letters, digits, dashes and underscores; start with a letter.";
  }
  if (pattern.startsWith("^(?:(?:[a-zA-Z]")) {
    return "Enter a valid URL, e.g. s3://bucket/path.";
  }
  return `Must match ${pattern}.`;
}

/**
 * Validate one value against its spec. Returns the first problem as a
 * user-facing message, or `undefined` when the value is acceptable.
 */
export function validateField(
  spec: FieldSpec,
  value: unknown,
): string | undefined {
  if (isEmpty(value)) return spec.required ? "Required." : undefined;

  switch (spec.kind) {
    case "string": {
      if (typeof value !== "string") return "Must be text.";
      if (spec.minLength !== undefined && value.length < spec.minLength) {
        return `Must be at least ${spec.minLength} characters.`;
      }
      if (spec.maxLength !== undefined && value.length > spec.maxLength) {
        return `Must be at most ${spec.maxLength} characters.`;
      }
      if (spec.pattern && !compile(spec.pattern).test(value)) {
        return patternMessage(spec.pattern);
      }
      return undefined;
    }
    case "enum":
      return spec.values.includes(value as string)
        ? undefined
        : `Must be one of ${spec.values.join(", ")}.`;
    case "boolean":
      return typeof value === "boolean" ? undefined : "Must be true or false.";
    case "number": {
      if (typeof value !== "number" || Number.isNaN(value)) {
        return "Must be a number.";
      }
      if (spec.minimum !== undefined && value < spec.minimum) {
        return `Must be at least ${spec.minimum}.`;
      }
      if (spec.maximum !== undefined && value > spec.maximum) {
        return `Must be at most ${spec.maximum}.`;
      }
      return undefined;
    }
    case "map": {
      if (typeof value !== "object") return "Must be key-value pairs.";
      const keys = Object.keys(value as object);
      if (keys.some((key) => key.trim() === "")) return "Keys can't be empty.";
      return undefined;
    }
    case "object": {
      if (typeof value !== "object") return "Must be an object.";
      const errors = validateRequest(
        spec.fields as Record<string, FieldSpec>,
        value as Record<string, unknown>,
      );
      const first = Object.entries(errors)[0];
      return first ? `${first[0]}: ${first[1]}` : undefined;
    }
  }
}

/** Per-field errors for a request body; empty when the body is valid. */
export type FieldErrors = Partial<Record<string, string>>;

/** Validate every field of `body` that has a spec. */
export function validateRequest(
  specs: Record<string, FieldSpec>,
  body: Record<string, unknown>,
): FieldErrors {
  const errors: FieldErrors = {};
  for (const [key, spec] of Object.entries(specs)) {
    const message = validateField(spec, body[key]);
    if (message) errors[key] = message;
  }
  return errors;
}

/** A human label for a field: its spec title, else its name in sentence case. */
export function fieldLabel(name: string, spec?: FieldSpec): string {
  const title = spec?.title;
  // Proto-derived titles are often a full sentence; prefer them only when short.
  if (title && title.length <= 40) return title;
  const words = name.replace(/_/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}
