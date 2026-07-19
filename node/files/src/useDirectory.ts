// React hook driving a paged directory listing. Lists the first page on mount
// (and whenever the path / pageSize / service change), accumulates entries across
// pages via `nextPageToken`, cancels the in-flight request on dep change and
// unmount, and re-renders as pages arrive.
//
// It is the files analog of @open-lakehouse/query's `usePreview`, but for a paged
// unary listing rather than a streaming Arrow run: a small internal store holds
// the accumulated entries + paging state, exposed through `useSyncExternalStore`
// so the (idle-until-`start()`) run can't emit before a subscriber is attached —
// the same StrictMode-safe, deferred-start discipline `usePreview` uses.

import { useEffect, useMemo, useSyncExternalStore } from "react";
import { useFilesService } from "./context";
import type { DirectoryEntry, FilesService } from "./types";

/** What `useDirectory` returns — a live view of the accumulated listing. */
export interface DirectoryState {
  /** Entries accumulated across all loaded pages, in listing order. */
  entries: DirectoryEntry[];
  /** True while a page request is in flight (initial load or `loadMore`). */
  isLoading: boolean;
  /** The terminal error of the last page request, or null. */
  error: Error | null;
  /** True when a `nextPageToken` remains — call `loadMore()` to fetch it. */
  hasMore: boolean;
  /** Fetch the next page and append its entries. No-op while loading or done. */
  loadMore(): void;
  /** Discard all pages and re-list from the first page. */
  reload(): void;
}

// Internal handle: owns the accumulated entries, paging cursor, a subscriber set,
// and the request lifecycle. A single monotonic `version` is the
// `useSyncExternalStore` snapshot; it bumps on every state change.
class DirectoryListing {
  private subscribers = new Set<() => void>();
  private controller = new AbortController();
  private _version = 0;
  private _entries: DirectoryEntry[] = [];
  private _loading = false;
  private _error: Error | null = null;
  private _nextToken: string | undefined;
  private _hasMore = true;
  private _active = false;
  private _started = false;

  constructor(
    private readonly svc: FilesService,
    private readonly path: string,
    private readonly pageSize?: number,
  ) {}

  // Begin the initial listing. Called from the hook's mount effect (after
  // subscribe is attached). StrictMode-safe: mount → cleanup → mount fires
  // start() → cancel() → start() on the same handle, so the second start() must
  // spin up a fresh run rather than no-op on the aborted controller.
  start(): void {
    if (this._active) return;
    this._active = true;
    // A restart (StrictMode's second mount) resets from scratch: fresh
    // controller + cleared accumulation so pages can't double-append.
    this.controller = new AbortController();
    this._entries = [];
    this._error = null;
    this._nextToken = undefined;
    this._hasMore = true;
    this._started = false;
    void this.fetchPage();
  }

  get version(): number {
    return this._version;
  }
  get entries(): DirectoryEntry[] {
    return this._entries;
  }
  get loading(): boolean {
    return this._loading;
  }
  get error(): Error | null {
    return this._error;
  }
  get hasMore(): boolean {
    return this._hasMore;
  }

  subscribe(cb: () => void): () => void {
    this.subscribers.add(cb);
    return () => this.subscribers.delete(cb);
  }

  cancel(): void {
    // Clearing `_active` lets a later start() (StrictMode's second mount) spin up
    // a fresh run instead of no-opping on the aborted controller.
    this._active = false;
    if (!this.controller.signal.aborted) this.controller.abort();
  }

  // Fetch the next page (initial or continuation) and append its entries. Guards
  // against concurrent / redundant requests: no-op while a page is in flight or
  // when there is nothing more to load (after at least the first page).
  loadMore(): void {
    if (this._loading) return;
    if (this._started && !this._hasMore) return;
    void this.fetchPage();
  }

  // Discard everything and re-list from the first page.
  reload(): void {
    if (!this.controller.signal.aborted) this.controller.abort();
    this.controller = new AbortController();
    this._entries = [];
    this._error = null;
    this._nextToken = undefined;
    this._hasMore = true;
    this._started = false;
    this._loading = false;
    void this.fetchPage();
  }

  private bump(): void {
    this._version += 1;
    for (const cb of this.subscribers) cb();
  }

  private async fetchPage(): Promise<void> {
    // Capture the controller for THIS request: cancel()/reload() may swap
    // `this.controller`, but this fetch must observe the signal it launched with.
    const { signal } = this.controller;
    this._loading = true;
    this._error = null;
    this.bump();
    try {
      const page = await this.svc.listDirectory(
        {
          path: this.path,
          maxResults: this.pageSize,
          pageToken: this._nextToken,
        },
        signal,
      );
      if (signal.aborted) return;
      this._started = true;
      this._entries = [...this._entries, ...page.entries];
      this._nextToken = page.nextPageToken;
      this._hasMore = page.nextPageToken != null && page.nextPageToken !== "";
    } catch (err) {
      // An abort is an intentional teardown, not a surfaced error.
      if (signal.aborted) return;
      this._started = true;
      this._error = err instanceof Error ? err : new Error(String(err));
      this._hasMore = false;
    } finally {
      // Only the currently-live request reports completion.
      if (signal === this.controller.signal) {
        this._loading = false;
        this.bump();
      }
    }
  }
}

/**
 * List the contents of `path` (a `/Volumes/...` directory), accumulating entries
 * across pages. Pass an explicit `service` (via context) — the hook reads it from
 * {@link useFilesService}. `opts.pageSize` bounds each page; `loadMore()` fetches
 * the next page until `hasMore` is false; `reload()` re-lists from scratch.
 */
export function useDirectory(
  path: string,
  opts?: { pageSize?: number },
): DirectoryState {
  const svc = useFilesService();
  const pageSize = opts?.pageSize;

  // One listing per (service, path, pageSize). Built in render (memoized) so the
  // subscribe target is stable for the initial commit's `useSyncExternalStore`.
  // Idle until `start()` in the mount effect below.
  const listing = useMemo(
    () => new DirectoryListing(svc, path, pageSize),
    [svc, path, pageSize],
  );

  // Start the initial listing and arrange cleanup — after the subscription is
  // committed. StrictMode-safe (see `DirectoryListing.start`).
  useEffect(() => {
    listing.start();
    return () => listing.cancel();
  }, [listing]);

  useSyncExternalStore(
    (cb) => listing.subscribe(cb),
    () => listing.version,
    () => listing.version,
  );

  return {
    entries: listing.entries,
    isLoading: listing.loading,
    error: listing.error,
    hasMore: listing.hasMore,
    loadMore: () => listing.loadMore(),
    reload: () => listing.reload(),
  };
}
