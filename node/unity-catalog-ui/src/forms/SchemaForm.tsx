// Schema-driven form renderer.
//
// A thin wrapper around @rjsf/core that renders JSON Schemas (generated from the
// Unity Catalog OpenAPI spec — see
// scripts/gen-form-schemas.mjs) using the app's shadcn
// primitives instead of rjsf's default unstyled HTML. The
// default submit button is suppressed; callers render their own submit button in
// a dialog footer and wire it to the form via `form={id}` + `type="submit"`.

import {
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
} from "@open-lakehouse/ui-kit";
import Form from "@rjsf/core";
import type {
  BaseInputTemplateProps,
  DescriptionFieldProps,
  FieldErrorProps,
  FieldTemplateProps,
  ObjectFieldTemplateProps,
  RegistryWidgetsType,
  RJSFSchema,
  RJSFValidationError,
  TemplatesType,
  UiSchema,
  WidgetProps,
} from "@rjsf/utils";
import { getDefaultFormState } from "@rjsf/utils";
import { customizeValidator } from "@rjsf/validator-ajv8";
import Ajv2020 from "ajv/dist/2020";
import { useState } from "react";

function FieldTemplate(props: FieldTemplateProps) {
  const {
    id,
    label,
    required,
    children,
    errors,
    description,
    hidden,
    displayLabel,
  } = props;
  // Hidden fields still validate. Their errors are surfaced by the form-level
  // alert in `SchemaForm`, since the field itself is never visible.
  if (hidden) return <div className="hidden">{children}</div>;
  return (
    <div className="space-y-1.5">
      {displayLabel && label ? (
        <Label htmlFor={id}>
          {label}
          {required ? <span className="text-destructive"> *</span> : null}
        </Label>
      ) : null}
      {children}
      {displayLabel ? description : null}
      {errors}
    </div>
  );
}

function ObjectFieldTemplate(props: ObjectFieldTemplateProps) {
  const { title, properties, idSchema } = props;
  const isRoot = idSchema.$id === "root";
  return (
    <div className="space-y-3">
      {!isRoot && title ? (
        <div className="text-xs font-medium text-foreground">{title}</div>
      ) : null}
      {properties.map((el) => (
        <div key={el.name}>{el.content}</div>
      ))}
    </div>
  );
}

function BaseInputTemplate(props: BaseInputTemplateProps) {
  const {
    id,
    value,
    type,
    placeholder,
    required,
    disabled,
    readonly,
    autofocus,
    onChange,
    onChangeOverride,
    onBlur,
    onFocus,
    options,
  } = props;
  return (
    <Input
      id={id}
      type={type === "number" ? "number" : "text"}
      value={value ?? ""}
      placeholder={placeholder}
      required={required}
      disabled={disabled || readonly}
      autoFocus={autofocus}
      onChange={
        onChangeOverride ??
        ((e) => {
          const next = e.target.value;
          onChange(next === "" ? (options.emptyValue ?? undefined) : next);
        })
      }
      onBlur={(e) => onBlur(id, e.target.value)}
      onFocus={(e) => onFocus(id, e.target.value)}
    />
  );
}

function DescriptionFieldTemplate(props: DescriptionFieldProps) {
  const { description } = props;
  if (!description) return null;
  return <p className="text-xs text-muted-foreground">{description}</p>;
}

function FieldErrorTemplate(props: FieldErrorProps) {
  const { errors = [] } = props;
  if (!errors.length) return null;
  return (
    <ul className="space-y-0.5">
      {errors.map((error, index) => (
        // rjsf supplies a stable, deduplicated list of errors per field.
        // biome-ignore lint/suspicious/noArrayIndexKey: errors have no stable id
        <li key={index} className="text-xs text-destructive">
          {error}
        </li>
      ))}
    </ul>
  );
}

function SelectWidget(props: WidgetProps) {
  const { id, value, options, onChange, placeholder, disabled, readonly } =
    props;
  const enumOptions = options.enumOptions ?? [];
  return (
    <Select
      value={value == null || value === "" ? undefined : String(value)}
      onValueChange={(next) => onChange(next)}
      disabled={disabled || readonly}
    >
      <SelectTrigger id={id}>
        <SelectValue placeholder={placeholder || "Select…"} />
      </SelectTrigger>
      <SelectContent>
        {enumOptions.map((opt) => (
          <SelectItem key={String(opt.value)} value={String(opt.value)}>
            {opt.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

function TextareaWidget(props: WidgetProps) {
  const {
    id,
    value,
    placeholder,
    disabled,
    readonly,
    autofocus,
    onChange,
    onBlur,
    onFocus,
    options,
  } = props;
  return (
    <Textarea
      id={id}
      value={value ?? ""}
      placeholder={placeholder}
      disabled={disabled || readonly}
      autoFocus={autofocus}
      rows={typeof options.rows === "number" ? options.rows : 3}
      onChange={(e) => {
        const next = e.target.value;
        onChange(next === "" ? (options.emptyValue ?? undefined) : next);
      }}
      onBlur={(e) => onBlur(id, e.target.value)}
      onFocus={(e) => onFocus(id, e.target.value)}
    />
  );
}

const templates: Partial<TemplatesType> = {
  FieldTemplate,
  ObjectFieldTemplate,
  BaseInputTemplate,
  DescriptionFieldTemplate,
  FieldErrorTemplate,
};

const widgets: RegistryWidgetsType = {
  SelectWidget,
  TextareaWidget,
};

// The form schemas are generated from protobuf as JSON Schema draft 2020-12
// (with `$defs`), so the validator must use the 2020-12 dialect. The protovalidate
// URI patterns rely on identity escapes that are invalid under JS unicode-mode
// regex, so disable `unicodeRegExp` to compile them.
export const validator = customizeValidator({
  AjvClass: Ajv2020,
  ajvOptionsOverrides: { unicodeRegExp: false },
});

/**
 * Validate form data against a schema the way the rendered form does on submit:
 * fill schema defaults first, then run Ajv. Returns the validation errors.
 */
export function validateFormData(
  schema: RJSFSchema,
  formData: unknown,
): RJSFValidationError[] {
  const filled = getDefaultFormState(
    validator,
    schema,
    formData as never,
    schema,
  );
  return validator.validateFormData(filled, schema).errors;
}

/**
 * The top-level property an error belongs to, e.g. `catalog_name` for
 * `.catalog_name`, or `aws_iam_role` for `.aws_iam_role.role_arn`.
 */
function errorField(error: RJSFValidationError): string | undefined {
  const path = error.property?.replace(/^\./, "") ?? "";
  const top = path.split(".")[0];
  if (top) return top;
  // Root-level errors (e.g. `required`) name the field in params.
  const params = error.params as { missingProperty?: string } | undefined;
  return params?.missingProperty;
}

/** True if the uiSchema hides `field` at the top level. */
function isHidden(uiSchema: UiSchema | undefined, field: string): boolean {
  const fieldUi = uiSchema?.[field] as { "ui:widget"?: string } | undefined;
  return fieldUi?.["ui:widget"] === "hidden";
}

/**
 * The errors no visible field can show: those on hidden fields and those at
 * the root (e.g. `additionalProperties`).
 */
export function invisibleErrors(
  errors: RJSFValidationError[],
  uiSchema: UiSchema | undefined,
): RJSFValidationError[] {
  return errors.filter((error) => {
    const field = errorField(error);
    return !field || isHidden(uiSchema, field);
  });
}

export interface SchemaFormProps<T> {
  /** DOM id of the rendered `<form>`; pair with a footer `<button form={id}>`. */
  id: string;
  schema: RJSFSchema;
  uiSchema?: UiSchema;
  formData?: T;
  disabled?: boolean;
  onChange?: (data: T) => void;
  onSubmit: (data: T) => void;
}

export function SchemaForm<T>({
  id,
  schema,
  uiSchema,
  formData,
  disabled,
  onChange,
  onSubmit,
}: SchemaFormProps<T>) {
  // Errors the user can't see next to a field: those on hidden fields and those
  // at the root (e.g. `additionalProperties`). Without this, a submit blocked by
  // such an error does nothing at all.
  const [formErrors, setFormErrors] = useState<string[]>([]);

  // Suppress rjsf's built-in submit button; the dialog footer owns submission.
  const mergedUiSchema: UiSchema = {
    ...uiSchema,
    "ui:submitButtonOptions": { norender: true, submitText: "", props: {} },
  };

  return (
    <>
      <Form
        id={id}
        schema={schema}
        uiSchema={mergedUiSchema}
        formData={formData}
        validator={validator}
        disabled={disabled}
        showErrorList={false}
        noHtml5Validate
        templates={templates}
        widgets={widgets}
        onChange={(e) => onChange?.(e.formData as T)}
        onError={(errors) => {
          const invisible = invisibleErrors(errors, uiSchema);
          if (invisible.length) {
            console.error("SchemaForm: submit blocked by", invisible);
          }
          setFormErrors(invisible.map((error) => error.stack));
        }}
        onSubmit={(e) => {
          setFormErrors([]);
          onSubmit(e.formData as T);
        }}
      />
      {formErrors.length ? (
        <ul role="alert" className="mt-3 space-y-0.5">
          {formErrors.map((message) => (
            <li key={message} className="text-xs text-destructive">
              {message}
            </li>
          ))}
        </ul>
      ) : null}
    </>
  );
}
