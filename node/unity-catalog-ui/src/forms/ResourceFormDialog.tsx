// Shared chrome for the create/edit resource dialogs: title, description, the
// form body, a form-level error region and the footer buttons.

import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@open-lakehouse/ui-kit";
import { type FormEvent, type ReactNode, useRef } from "react";
import { FormError } from "./fields.js";

export function ResourceFormDialog({
  title,
  description,
  children,
  formError,
  loadError,
  pending,
  canSubmit = true,
  submitLabel,
  pendingLabel = "Saving…",
  onSubmit,
  onClose,
  footer,
}: {
  title: string;
  description?: ReactNode;
  children: ReactNode;
  /** A request failure no field claimed. */
  formError?: string;
  /** The resource couldn't be loaded; the form is not shown. */
  loadError?: string;
  pending?: boolean;
  /** False while required context (e.g. a parent) is missing. */
  canSubmit?: boolean;
  submitLabel: string;
  pendingLabel?: string;
  onSubmit: (e: FormEvent) => void;
  onClose: () => void;
  /** Replaces the default Cancel / submit buttons. */
  footer?: ReactNode;
}) {
  const formRef = useRef<HTMLFormElement>(null);

  function submit(e: FormEvent) {
    // Dialogs portal out of the DOM tree, but React events still bubble along
    // the component tree: a dialog opened from inside this form (e.g. "New
    // storage credential") would otherwise submit this form too.
    e.stopPropagation();
    e.preventDefault();
    onSubmit(e);
    // After React renders the errors, move focus to the first invalid field.
    requestAnimationFrame(() => {
      formRef.current
        ?.querySelector<HTMLElement>('[aria-invalid="true"]')
        ?.focus();
    });
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <form ref={formRef} onSubmit={submit} noValidate>
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            {description ? (
              <DialogDescription>{description}</DialogDescription>
            ) : null}
          </DialogHeader>

          <div className="max-h-[65vh] space-y-4 overflow-y-auto px-5 py-4">
            {loadError ? <FormError message={loadError} /> : children}
            <FormError message={formError} />
          </div>

          <DialogFooter>
            {footer ?? (
              <>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={onClose}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  size="sm"
                  disabled={pending || !canSubmit || !!loadError}
                >
                  {pending ? pendingLabel : submitLabel}
                </Button>
              </>
            )}
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
