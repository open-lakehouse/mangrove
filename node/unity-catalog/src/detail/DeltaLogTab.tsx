// The Delta-log explorer surface of TableDetail. Additive and gated: it renders
// nothing unless (1) a log-query runner has been registered (the standalone
// build registers none — see @open-lakehouse/log-query) and (2) the service
// supports this table (Delta only). So the standalone website shows no Delta-log
// tab, and a host / the future wasm engine lights it up by registering a runner.
//
// It reuses the exact query-preview machinery: a swappable log-query service
// streams reconciled-log rows (from the async-native ReconciledLogProvider) as
// Arrow IPC into the same <DataGrid> the table preview uses. The table context
// (fullName) comes from TableDetail — this is where we already know which log to
// load, so no separate navigation is needed.
//
// The `useLogPreview` hook must run unconditionally, so the gate lives in
// `DeltaLogTab` and the hook lives in the inner `DeltaLogGrid`.

import { DataGrid } from "@open-lakehouse/data-grid";
import {
  hasLogQueryRunner,
  type LogKind,
  useLogPreview,
  useLogQueryService,
} from "@open-lakehouse/log-query";
import { Tabs, TabsList, TabsTrigger } from "@open-lakehouse/ui-kit";
import { ScrollText } from "lucide-react";
import { useState } from "react";

export function DeltaLogTab({
  fullName,
  format,
  tableType,
}: {
  fullName: string;
  format?: string;
  tableType?: string;
}) {
  const svc = useLogQueryService();
  // Which log surface is shown: the surviving files (`reconciled`) or the full
  // reconciled action stream (`actions`). Toggling re-runs the query.
  const [kind, setKind] = useState<LogKind>("reconciled");

  // Gate before touching the hook: a registered runner + capability. TableDetail
  // gates the trigger on the same conditions, so this is defence in depth.
  if (!hasLogQueryRunner()) return null;
  if (!svc.supports({ format, tableType })) return null;

  return (
    <div>
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          <ScrollText className="h-4 w-4" />
          Delta log
        </div>
        <Tabs value={kind} onValueChange={(v) => setKind(v as LogKind)}>
          <TabsList>
            <TabsTrigger value="reconciled">Reconciled</TabsTrigger>
            <TabsTrigger value="actions">Actions</TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
      <DeltaLogGrid target={fullName} kind={kind} />
    </div>
  );
}

function DeltaLogGrid({ target, kind }: { target: string; kind: LogKind }) {
  const { store, version, running, error } = useLogPreview({
    target,
    limit: 100,
    kind,
  });

  if (error) {
    return <p className="text-sm text-destructive">{error.message}</p>;
  }

  return (
    <>
      {running && store.rowCount === 0 ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span className="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-muted border-t-primary" />
          Loading Delta log…
        </div>
      ) : (
        <DataGrid store={store} version={version} running={running} />
      )}
    </>
  );
}
