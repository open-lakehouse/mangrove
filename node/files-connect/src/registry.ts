// Pluggable transport registry for the files ConnectRPC client — the single seam
// that lets a host route the `FilesService` client at a wasm backend (or anything
// else) WITHOUT the consumer depending on that backend. Mirrors hydrofoil's
// node/ui/src/lib/client/registry.ts, scoped to the one files service.
//
// Until a host registers one, the client transport throws `NoFilesTransportError`
// on every call, so an accidentally-constructed client fails loudly rather than
// hanging. `@open-lakehouse/files-wasm` registers the worker-backed transport at
// startup.

import type { Transport } from "@connectrpc/connect";

/** Thrown by the default (unregistered) transport. A registered transport never
 *  throws it; consumers surface it as "no files backend wired". */
export class NoFilesTransportError extends Error {
  constructor() {
    super(
      "No files transport registered. The files ConnectRPC client needs a " +
        "transport installed via registerFilesTransport (e.g. the wasm " +
        "worker-backed transport from @open-lakehouse/files-wasm).",
    );
    this.name = "NoFilesTransportError";
  }
}

const noopTransport: Transport = {
  unary() {
    return Promise.reject(new NoFilesTransportError());
  },
  stream() {
    return Promise.reject(new NoFilesTransportError());
  },
};

let current: Transport = noopTransport;

/** Install the files transport. Hosts call this once, before the UI bootstraps
 *  (late binding tolerates any order relative to client construction). */
export function registerFilesTransport(transport: Transport): void {
  current = transport;
}

/** The transport currently in effect (the registered one, or the throwing default). */
export function getFilesTransport(): Transport {
  return current;
}

/** True once a real transport has been registered — the probe the seam reads so
 *  `canWrite()` only turns true when the write-capable backend is wired. */
export function hasFilesTransport(): boolean {
  return current !== noopTransport;
}

/** Stable, late-binding transport handed to the files client. Each method
 *  dereferences `current` on every call, so registration order relative to client
 *  construction never matters. */
export const clientTransport: Transport = {
  unary(...args) {
    return current.unary(...args);
  },
  stream(...args) {
    return current.stream(...args);
  },
};
