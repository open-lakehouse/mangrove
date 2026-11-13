// Field components bound to a `ResourceForm`. Each one reads its spec, so the
// label, required marker, help text, constraints and errors all come from the
// OpenAPI spec. Forms pick and arrange fields; they don't restate rules.

import {
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
} from "@open-lakehouse/ui-kit";
import { Plus, X } from "lucide-react";
import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { type FieldSpec, fieldLabel, validateField } from "./spec.js";
import type { ResourceForm } from "./useResourceForm.js";

// biome-ignore lint/suspicious/noExplicitAny: forms of any request body
type AnyForm = ResourceForm<any>;

interface FieldProps {
  form: AnyForm;
  name: string;
  /** Overrides the spec-derived label. */
  label?: string;
  /** Overrides the spec description as help text; `null` hides it. */
  help?: ReactNode | null;
  placeholder?: string;
  autoFocus?: boolean;
  disabled?: boolean;
  /** Marks the field required in the UI when the spec doesn't. */
  required?: boolean;
}

function specOf(form: AnyForm, name: string): FieldSpec | undefined {
  return form.specs[name];
}

/** Label, help text and error around a control. */
export function FieldShell({
  id,
  label,
  required,
  help,
  error,
  children,
}: {
  id: string;
  label: string;
  required?: boolean;
  help?: ReactNode;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>
        {label}
        {required ? <span className="text-destructive"> *</span> : null}
      </Label>
      {children}
      {error ? (
        <p id={`${id}-error`} className="text-xs text-destructive">
          {error}
        </p>
      ) : help ? (
        <p id={`${id}-help`} className="text-xs text-muted-foreground">
          {help}
        </p>
      ) : null}
    </div>
  );
}

function useFieldMeta(props: FieldProps) {
  const id = useId();
  const spec = specOf(props.form, props.name);
  const error = props.form.error(props.name);
  return {
    id,
    spec,
    error,
    label: props.label ?? fieldLabel(props.name, spec),
    required: props.required ?? spec?.required,
    help: props.help === null ? undefined : (props.help ?? spec?.description),
    describedBy: error ? `${id}-error` : `${id}-help`,
  };
}

/** A single-line text field. */
export function TextField(props: FieldProps & { mono?: boolean }) {
  const { form, name } = props;
  const meta = useFieldMeta(props);
  const value = form.values[name];
  return (
    <FieldShell {...meta}>
      <Input
        id={meta.id}
        name={name}
        value={typeof value === "string" ? value : ""}
        placeholder={props.placeholder}
        autoFocus={props.autoFocus}
        disabled={props.disabled}
        aria-invalid={!!meta.error}
        aria-describedby={meta.describedBy}
        className={props.mono ? "font-mono" : undefined}
        onChange={(e) =>
          form.set(name, e.target.value === "" ? undefined : e.target.value)
        }
        onBlur={() => form.touch(name)}
      />
    </FieldShell>
  );
}

/**
 * A securable name. Names are lowercase identifiers, so typed capitals and
 * spaces are lowercased and turned into underscores as the user types.
 */
export function NameField(props: FieldProps) {
  const { form, name } = props;
  const meta = useFieldMeta(props);
  const value = form.values[name];
  return (
    <FieldShell
      {...meta}
      help={
        props.help === null
          ? undefined
          : (props.help ?? "Lowercase letters, digits and underscores.")
      }
    >
      <Input
        id={meta.id}
        name={name}
        value={typeof value === "string" ? value : ""}
        placeholder={props.placeholder}
        autoFocus={props.autoFocus}
        disabled={props.disabled}
        autoComplete="off"
        spellCheck={false}
        aria-invalid={!!meta.error}
        aria-describedby={meta.describedBy}
        className="font-mono"
        onChange={(e) => {
          const next = e.target.value.toLowerCase().replace(/\s/g, "_");
          form.set(name, next === "" ? undefined : next);
        }}
        onBlur={() => form.touch(name)}
      />
    </FieldShell>
  );
}

/** A multi-line free-text field, e.g. `comment`. */
export function CommentField(props: Partial<FieldProps> & { form: AnyForm }) {
  const full: FieldProps = {
    name: "comment",
    label: "Comment",
    placeholder: "What is this for?",
    help: null,
    ...props,
  };
  const { form, name } = full;
  const meta = useFieldMeta(full);
  const value = form.values[name];
  return (
    <FieldShell {...meta}>
      <Textarea
        id={meta.id}
        name={name}
        rows={2}
        value={typeof value === "string" ? value : ""}
        placeholder={full.placeholder}
        disabled={full.disabled}
        aria-invalid={!!meta.error}
        aria-describedby={meta.describedBy}
        onChange={(e) =>
          form.set(name, e.target.value === "" ? undefined : e.target.value)
        }
        onBlur={() => form.touch(name)}
      />
    </FieldShell>
  );
}

/** A choice among the spec's enum values. */
export function EnumField(
  props: FieldProps & {
    /** Labels for enum values; defaults to the value itself. */
    labels?: Record<string, string>;
  },
) {
  const { form, name } = props;
  const meta = useFieldMeta(props);
  const values = meta.spec?.kind === "enum" ? meta.spec.values : [];
  const value = form.values[name];
  return (
    <FieldShell {...meta}>
      <Select
        value={typeof value === "string" && value ? value : undefined}
        onValueChange={(next) => {
          // Radix reports "" for a value outside the options; not a choice.
          if (!next) return;
          form.set(name, next);
          form.touch(name);
        }}
        disabled={props.disabled}
      >
        <SelectTrigger
          id={meta.id}
          aria-invalid={!!meta.error}
          aria-describedby={meta.describedBy}
        >
          <SelectValue placeholder={props.placeholder ?? "Select…"} />
        </SelectTrigger>
        <SelectContent>
          {values.map((v) => (
            <SelectItem key={v} value={v}>
              {props.labels?.[v] ?? v}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </FieldShell>
  );
}

/** An on/off flag, with its help text beside the switch. */
export function SwitchField(props: FieldProps) {
  const { form, name } = props;
  const meta = useFieldMeta(props);
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="space-y-0.5">
        <Label htmlFor={meta.id}>{meta.label}</Label>
        {meta.help ? (
          <p className="text-xs text-muted-foreground">{meta.help}</p>
        ) : null}
      </div>
      <Switch
        id={meta.id}
        checked={form.values[name] === true}
        disabled={props.disabled}
        onCheckedChange={(checked) => form.set(name, checked)}
      />
    </div>
  );
}

type Row = { id: number; key: string; value: string };

let nextRowId = 0;

function rowsOf(value: unknown): Row[] {
  if (!value || typeof value !== "object") return [];
  return Object.entries(value as Record<string, string>).map(([key, v]) => ({
    id: nextRowId++,
    key,
    value: v,
  }));
}

/**
 * A `map<string, string>` field as editable key-value rows. Rows with an empty
 * key are kept while editing but not sent. Duplicate keys are flagged.
 */
export function PropertiesField(
  props: Partial<FieldProps> & { form: AnyForm },
) {
  const full: FieldProps = {
    name: "properties",
    label: "Properties",
    help: null,
    ...props,
  };
  const { form, name } = full;
  const meta = useFieldMeta(full);
  const current = form.values[name];
  const [rows, setRows] = useState<Row[]>(() => rowsOf(current));

  // Follow external changes (e.g. an edit form seeded after load), but not our
  // own: rows with an empty key don't show in the emitted map.
  const serialized = JSON.stringify(current ?? {});
  const emitted = useRef(serialized);
  useEffect(() => {
    if (serialized === emitted.current) return;
    emitted.current = serialized;
    setRows(rowsOf(current));
  }, [serialized, current]);

  function update(next: Row[]) {
    setRows(next);
    const map: Record<string, string> = {};
    for (const row of next) {
      const key = row.key.trim();
      if (key) map[key] = row.value;
    }
    emitted.current = JSON.stringify(map);
    form.set(name, Object.keys(map).length ? map : undefined);
  }

  const keys = rows.map((r) => r.key.trim()).filter(Boolean);
  const duplicate = keys.find((k, i) => keys.indexOf(k) !== i);

  return (
    <FieldShell
      {...meta}
      error={
        meta.error ??
        (duplicate
          ? `Duplicate key "${duplicate}": the last value wins.`
          : undefined)
      }
    >
      <div id={meta.id} className="space-y-1.5">
        {rows.map((row) => (
          <div key={row.id} className="flex items-center gap-1.5">
            <Input
              aria-label="Key"
              value={row.key}
              placeholder="key"
              className="font-mono"
              disabled={full.disabled}
              onChange={(e) =>
                update(
                  rows.map((r) =>
                    r.id === row.id ? { ...r, key: e.target.value } : r,
                  ),
                )
              }
            />
            <Input
              aria-label="Value"
              value={row.value}
              placeholder="value"
              disabled={full.disabled}
              onChange={(e) =>
                update(
                  rows.map((r) =>
                    r.id === row.id ? { ...r, value: e.target.value } : r,
                  ),
                )
              }
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              aria-label={`Remove ${row.key || "property"}`}
              disabled={full.disabled}
              onClick={() => update(rows.filter((r) => r.id !== row.id))}
            >
              <X className="h-3.5 w-3.5" />
            </Button>
          </div>
        ))}
        <Button
          type="button"
          variant="link"
          size="sm"
          className="h-auto p-0 text-xs"
          disabled={full.disabled}
          onClick={() =>
            setRows([...rows, { id: nextRowId++, key: "", value: "" }])
          }
        >
          <Plus className="h-3.5 w-3.5" />
          Add property
        </Button>
      </div>
    </FieldShell>
  );
}

/**
 * The text fields of a nested object field (e.g. a credential's
 * `aws_iam_role`), each validated against its own sub-spec. Fields listed in
 * `advanced` are tucked under a disclosure; fields in `secret` are masked.
 */
export function ObjectFields({
  form,
  name,
  labels = {},
  placeholders = {},
  advanced = [],
  secret = [],
  disabled,
}: {
  form: AnyForm;
  name: string;
  labels?: Record<string, string>;
  placeholders?: Record<string, string>;
  advanced?: readonly string[];
  secret?: readonly string[];
  disabled?: boolean;
}) {
  const idBase = useId();
  const [showAdvanced, setShowAdvanced] = useState(false);
  const spec = specOf(form, name);
  if (spec?.kind !== "object") return null;

  const fields = Object.entries(spec.fields as Record<string, FieldSpec>);
  const value = (form.values[name] ?? {}) as Record<string, unknown>;
  // Errors show once the form shows the parent's error (touched or submitted).
  const showErrors = form.error(name) !== undefined;

  function set(sub: string, next: string) {
    const updated = { ...value, [sub]: next === "" ? undefined : next };
    form.set(name, updated);
  }

  const render = ([sub, subSpec]: [string, FieldSpec]) => {
    const id = `${idBase}-${sub}`;
    const subValue = value[sub];
    const error = showErrors ? validateField(subSpec, subValue) : undefined;
    return (
      <FieldShell
        key={sub}
        id={id}
        label={labels[sub] ?? fieldLabel(sub, subSpec)}
        required={subSpec.required}
        help={subSpec.description}
        error={error}
      >
        <Input
          id={id}
          type={secret.includes(sub) ? "password" : "text"}
          autoComplete="off"
          value={typeof subValue === "string" ? subValue : ""}
          placeholder={placeholders[sub]}
          disabled={disabled}
          aria-invalid={!!error}
          className="font-mono"
          onChange={(e) => set(sub, e.target.value)}
          onBlur={() => form.touch(name)}
        />
      </FieldShell>
    );
  };

  const primary = fields.filter(([sub]) => !advanced.includes(sub));
  const extra = fields.filter(([sub]) => advanced.includes(sub));

  if (fields.length === 0) {
    return (
      <p className="text-xs text-muted-foreground">
        Nothing to configure: the server manages this credential.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      {primary.map(render)}
      {extra.length > 0 && (
        <div className="space-y-3">
          <button
            type="button"
            className="text-xs font-medium text-muted-foreground hover:text-foreground"
            aria-expanded={showAdvanced}
            onClick={() => setShowAdvanced((open) => !open)}
          >
            {showAdvanced ? "Hide advanced settings" : "Advanced settings"}
          </button>
          {showAdvanced && extra.map(render)}
        </div>
      )}
    </div>
  );
}

/** The form-level error region: server errors no field claimed. */
export function FormError({ message }: { message?: string }) {
  if (!message) return null;
  return (
    <div
      role="alert"
      className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive"
    >
      {message}
    </div>
  );
}
