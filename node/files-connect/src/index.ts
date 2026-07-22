// `@open-lakehouse/files-connect`: the write-capable ConnectRPC surface for the
// in-browser volume-files engine.
//
// It wraps the vendored `portal.files.v1.FilesService` proto contract in a
// ConnectRPC client (`createFilesClient`) over a pluggable, late-binding
// transport (`registerFilesTransport`). The write-capable transport is the
// wasm-backed one (`createWasmFilesTransport`), which mirrors hydrofoil's Tauri
// backend split: unary metadata RPCs go through the in-wasm connect dispatcher as
// binary proto, and file bytes bypass it via native calls. `@open-lakehouse/
// files-wasm` wires this to a Web Worker and adapts the client to the
// `@open-lakehouse/files` seam; the (separately-updated) editor consumes the
// client later.

export { createFilesClient, type FilesClient } from "./client";
// Re-export the generated service descriptor + message schemas so consumers
// build request messages (`create(GetFileMetadataRequestSchema, …)`) without
// reaching into the generated tree directly.
export * from "./gen/portal/files/v1/svc_pb";
export {
  clientTransport,
  getFilesTransport,
  hasFilesTransport,
  NoFilesTransportError,
  registerFilesTransport,
} from "./registry";
export {
  createWasmFilesTransport,
  type FilesBackend,
} from "./transport";
