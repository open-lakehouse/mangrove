// React hook driving a table preview. Starts a run on mount (and whenever the
// table / limit / columns / service change), cancels the previous run and on
// unmount, and re-renders as chunks stream in via `useSyncExternalStore` over the
// handle's `subscribe`.

import { useEffect, useMemo, useSyncExternalStore } from "react";
import { useQueryService } from "./context";
import type { PreviewHandle, QueryService } from "./types";

/** What `usePreview` returns — a live view of the current run. */
export interface PreviewState {
  /** The grid store; pass to `<DataGrid store={...}>`. */
  store: PreviewHandle["store"];
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
 * Preview `tableFullName`, streaming rows into a grid store. Pass an explicit
 * `service` to override the one from context (tests / composition).
 */
export function usePreview(
  req: { tableFullName: string; limit?: number; columns?: string[] },
  service?: QueryService,
): PreviewState {
  const ctxService = useQueryService();
  const svc = service ?? ctxService;

  // Stabilize the columns array by identity via a string key, so a caller
  // passing a fresh `["a","b"]` literal each render doesn't restart the preview.
  const columnsKey = req.columns?.join(",") ?? "";
  const columns = useMemo(
    () => (columnsKey ? columnsKey.split(",") : undefined),
    [columnsKey],
  );

  // One handle per (service, table, limit, columns). Recreating it here (in
  // render, memoized) rather than in an effect means the store/subscribe are
  // stable for the initial commit's `useSyncExternalStore`. The handle is built
  // idle: it doesn't start streaming until `start()` below.
  const { tableFullName, limit } = req;
  const handle = useMemo(
    () => svc.preview({ tableFullName, limit, columns }),
    [svc, tableFullName, limit, columns],
  );

  // Start the run and arrange cleanup. This effect runs *after* the
  // `useSyncExternalStore` subscription is committed, so no chunk can be
  // appended before a subscriber exists to observe its bump. Under React
  // StrictMode this fires start() → cancel() → start() on the same handle;
  // `PreviewRun` handles that by re-running on the second start() with a fresh
  // AbortController (a plain "already started/aborted" guard would leave the run
  // dead and the grid empty until a table switch built a new handle).
  useEffect(() => {
    handle.start();
    return () => handle.cancel();
  }, [handle]);

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
