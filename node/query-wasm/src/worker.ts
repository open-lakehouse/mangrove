// The wasm worker: hosts `UcQueryEngine` (crates/query-wasm) off the main
// thread — the kernel's inline-executor bursts run synchronously against primed
// data and would jank the UI if run on the main thread.
//
// "query-wasm-pkg" is a BARE specifier for the gitignored wasm-bindgen output
// of `just build-query-wasm` (crates/query-wasm/pkg/query_wasm.js): typed by
// the ambient declaration in ./pkg.d.ts and resolved by the vite alias in
// node/app/vite.config.ts — only in wasm-enabled builds (default builds alias
// the whole package to ./stub.ts and never bundle this worker).
import init, { UcQueryEngine } from "query-wasm-pkg";
import type { RunMessage, WorkerResponse } from "./protocol";

const post = (message: WorkerResponse, transfer: Transferable[] = []) =>
  (self as unknown as Worker).postMessage(message, transfer);

self.onmessage = async (event: MessageEvent<RunMessage>) => {
  const request = event.data;
  if (request.type !== "run") return;
  try {
    await init();
    const engine = new UcQueryEngine(request.baseUrl, {
      authToken: request.authToken,
    });
    const stats = await engine.runQuery(
      request.sql,
      {
        limit: request.limit,
        catalog: request.catalog,
        schema: request.schema,
      },
      (ipc, numRows) => {
        // The Uint8Array views wasm memory; copy before transferring.
        const copy = ipc.slice();
        post({ type: "chunk", ipc: copy, numRows }, [copy.buffer]);
      },
    );
    post({ type: "done", stats });
  } catch (error) {
    const err = error as { message?: unknown; code?: unknown };
    post({
      type: "error",
      message: typeof err?.message === "string" ? err.message : String(error),
      code: typeof err?.code === "string" ? err.code : "FAILED",
    });
  }
};
