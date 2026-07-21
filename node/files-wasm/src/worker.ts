// The wasm worker: hosts `UcFilesEngine` (crates/query-wasm) off the main
// thread. Metadata RPCs dispatch through the engine's `connectUnary` (the in-wasm
// connect Router) as binary proto; file bytes stream through `readFileBytes` /
// `writeFileBytes` — kept off the UI thread to match the query worker and to
// avoid janking on large transfers.
//
// "query-wasm-pkg" is a BARE specifier for the gitignored wasm-bindgen output of
// `just build-query-wasm` (crates/query-wasm/pkg/query_wasm.js) — the SAME
// artifact @open-lakehouse/query-wasm imports; the files engine ships alongside
// the query engine. Typed by the ambient declaration in ./pkg.d.ts and resolved
// by the vite alias in node/app/vite.config.ts — only in wasm-enabled builds
// (default builds alias the whole package to ./stub.ts and never bundle this
// worker).
import init, { UcFilesEngine } from "query-wasm-pkg";
import type { WorkerRequest, WorkerResponse } from "./protocol";

const post = (message: WorkerResponse, transfer: Transferable[] = []) =>
  (self as unknown as Worker).postMessage(message, transfer);

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;
  if (
    request.type !== "connectUnary" &&
    request.type !== "read" &&
    request.type !== "write"
  ) {
    return;
  }
  try {
    await init();
    const engine = new UcFilesEngine(request.baseUrl, {
      authToken: request.authToken,
    });
    switch (request.type) {
      case "connectUnary": {
        const responseBytes = await engine.connectUnary(
          request.path,
          request.requestBytes,
        );
        // The response views wasm memory; copy before transferring.
        const copy = responseBytes.slice();
        post({ type: "unary", responseBytes: copy }, [copy.buffer]);
        break;
      }
      case "read": {
        // Each Uint8Array views wasm memory; copy before transferring.
        const onChunk = (bytes: Uint8Array) => {
          const copy = bytes.slice();
          post({ type: "chunk", bytes: copy }, [copy.buffer]);
        };
        await engine.readFileBytes(
          request.path,
          { offset: request.offset, length: request.length },
          onChunk,
        );
        break;
      }
      case "write": {
        const result = await engine.writeFileBytes(
          request.path,
          request.bytes,
          {
            contentType: request.contentType,
            ifMatchEtag: request.ifMatchEtag,
          },
        );
        post({ type: "writeResult", result });
        break;
      }
    }
    post({ type: "done" });
  } catch (error) {
    const err = error as { message?: unknown; code?: unknown };
    post({
      type: "error",
      message: typeof err?.message === "string" ? err.message : String(error),
      code: typeof err?.code === "string" ? err.code : "FAILED",
    });
  }
};
