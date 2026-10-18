// The data-preview surface of TableDetail (its own "Preview" tab). Additive and
// dark-launched: it's shown only when (1) the preview feature flag is on, (2) a
// query runner has been registered (the standalone build registers none — see
// @open-lakehouse/query), and (3) the service supports this table. So the
// standalone website ships with no preview, and a host / the future wasm engine
// lights it up by registering a runner. `useTablePreviewVisible` exposes that
// gate so TableDetail can hide the tab trigger too (no empty, inert pane).
//
// The `usePreview` hook must run unconditionally, so the gate lives in
// `TablePreview` and the hook lives in the inner `PreviewGrid`.

import { DataGrid } from "@open-lakehouse/data-grid";
import {
  hasQueryRunner,
  usePreview,
  useQueryService,
} from "@open-lakehouse/query";
import { Table2 } from "lucide-react";

// Vite statically replaces import.meta.env.*; undefined/"false" → off.
const PREVIEW_ENABLED = import.meta.env.VITE_ENABLE_PREVIEW === "true";

/** Whether the preview tab should be offered for this table: feature flag +
 *  a registered runner + the runner's capability probe. A hook because the
 *  capability check reads the query service from context. */
export function useTablePreviewVisible({
  format,
  tableType,
}: {
  format?: string;
  tableType?: string;
}): boolean {
  const svc = useQueryService();
  if (!PREVIEW_ENABLED || !hasQueryRunner()) return false;
  return svc.supports({ format, tableType });
}

export function TablePreview({
  fullName,
  format,
  tableType,
}: {
  fullName: string;
  format?: string;
  tableType?: string;
}) {
  // Defence in depth: the tab trigger is gated on useTablePreviewVisible, but
  // guard here too so an unconditional render never touches the hook when off.
  const visible = useTablePreviewVisible({ format, tableType });
  if (!visible) return null;

  return (
    <div>
      <div className="mb-2 flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        <Table2 className="h-4 w-4" />
        Data preview
      </div>
      <PreviewGrid fullName={fullName} />
    </div>
  );
}

function PreviewGrid({ fullName }: { fullName: string }) {
  const { store, version, running, error } = usePreview({
    tableFullName: fullName,
    limit: 100,
  });

  if (error) {
    return <p className="text-sm text-destructive">{error.message}</p>;
  }

  return (
    <>
      {running && store.rowCount === 0 ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span className="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-muted border-t-primary" />
          Loading preview…
        </div>
      ) : (
        <DataGrid store={store} version={version} running={running} />
      )}
    </>
  );
}
