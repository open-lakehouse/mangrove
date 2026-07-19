// The wasm worker: hosts `UcFilesEngine` (crates/query-wasm) off the main
// thread. Listing issues native cloud list REST calls and parses the response;
// reading streams file bytes — kept off the UI thread to match the query worker
// and to avoid janking on large reads.
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
    request.type !== "list" &&
    request.type !== "read" &&
    request.type !== "stat"
  ) {
    return;
  }
  try {
    await init();
    const engine = new UcFilesEngine(request.baseUrl, {
      authToken: request.authToken,
    });
    switch (request.type) {
      case "list": {
        const page = await engine.listDirectory(request.path, {
          maxResults: request.maxResults,
          pageToken: request.pageToken,
        });
        post({ type: "page", page });
        break;
      }
      case "stat": {
        const meta = await engine.stat(request.path);
        post({ type: "meta", meta });
        break;
      }
      case "read": {
        // Each Uint8Array views wasm memory; copy before transferring.
        const onChunk = (bytes: Uint8Array) => {
          const copy = bytes.slice();
          post({ type: "chunk", bytes: copy }, [copy.buffer]);
        };
        await engine.readFile(
          request.path,
          { offset: request.offset, length: request.length },
          onChunk,
        );
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
