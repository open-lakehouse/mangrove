// React hook driving a reconciled-Delta-log preview. Starts a run on mount (and
// whenever the target / limit / service change), cancels the previous run and on
// unmount, and re-renders as chunks stream in via `useSyncExternalStore` over the
// handle's `subscribe`. Mirrors @open-lakehouse/query's usePreview.ts.

import { useEffect, useMemo, useSyncExternalStore } from "react";
import { useLogQueryService } from "./context";
import type { LogKind } from "./runner";
import type { LogPreviewHandle, LogQueryService } from "./types";

/** What `useLogPreview` returns — a live view of the current run. */
export interface LogPreviewState {
  /** The grid store; pass to `<DataGrid store={...}>`. */
  store: LogPreviewHandle["store"];
  /** Snapshot version — pass to `<DataGrid version={...}>` to drive re-renders. */
  version: number;
  /** True while the stream is open. */
  running: boolean;
  /** Terminal error, or null. */
  error: Error | null;
  /** Abort the current run. */
  cancel(): void;
}

/**
 * Explore the reconciled Delta log of `target`, streaming rows into a grid
 * store. Pass an explicit `service` to override the one from context (tests /
 * composition).
 */
export function useLogPreview(
  req: { target: string; limit?: number; kind?: LogKind },
  service?: LogQueryService,
): LogPreviewState {
  const ctxService = useLogQueryService();
  const svc = service ?? ctxService;

  // One handle per (service, target, limit, kind). Recreating it here (in
  // render, memoized) rather than in an effect means the store/subscribe are
  // stable for the initial commit's `useSyncExternalStore`; adding `kind` to the
  // deps means toggling reconciled/actions starts a fresh run.
  const { target, limit, kind } = req;
  const handle = useMemo(
    () => svc.preview({ target, limit, kind }),
    [svc, target, limit, kind],
  );

  // Cancel the run when the handle is replaced or the component unmounts.
  useEffect(() => () => handle.cancel(), [handle]);

  // Re-render whenever the handle bumps; snapshot is the monotonic version.
  useSyncExternalStore(
    (cb) => handle.subscribe(cb),
    () => handle.version,
    () => handle.version,
  );

  return {
    store: handle.store,
    version: handle.version,
    running: handle.running,
    error: handle.error,
    cancel: () => handle.cancel(),
  };
}
