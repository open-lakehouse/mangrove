// The data-preview section of TableDetail. Additive and dark-launched: it
// renders nothing unless (1) the preview feature flag is on, (2) a query runner
// has been registered (the standalone build registers none — see
// @open-lakehouse/query), and (3) the service supports this table. So the
// standalone website ships with no preview shown, and a host / the future wasm
// engine lights it up by registering a runner.
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

export function TablePreview({
  fullName,
  format,
  tableType,
}: {
  fullName: string;
  format?: string;
  tableType?: string;
}) {
  const svc = useQueryService();

  // Gate before touching the hook: flag + a registered runner + capability.
  if (!PREVIEW_ENABLED || !hasQueryRunner()) return null;
  if (!svc.supports({ format, tableType })) return null;

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
