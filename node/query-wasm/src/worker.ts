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
import type { LogRunMessage, RunMessage, WorkerResponse } from "./protocol";

const post = (message: WorkerResponse, transfer: Transferable[] = []) =>
  (self as unknown as Worker).postMessage(message, transfer);

self.onmessage = async (event: MessageEvent<RunMessage | LogRunMessage>) => {
  const request = event.data;
  if (request.type !== "run" && request.type !== "run-log") return;
  try {
    await init();
    const engine = new UcQueryEngine(request.baseUrl, {
      authToken: request.authToken,
    });
    // The Uint8Array views wasm memory; copy before transferring.
    const onBatch = (ipc: Uint8Array, numRows: number) => {
      const copy = ipc.slice();
      post({ type: "chunk", ipc: copy, numRows }, [copy.buffer]);
    };
    const stats =
      request.type === "run"
        ? await engine.runQuery(
            request.sql,
            {
              limit: request.limit,
              catalog: request.catalog,
              schema: request.schema,
            },
            onBatch,
          )
        : // The log surface is addressed by `target` + `kind`; the engine
          // synthesizes its own `delta_*_log('target')` query, so the `sql`
          // parameter is vestigial (kept in the wasm-bindgen signature to avoid a
          // crate rebuild) — pass an empty string.
          await engine.runLogQuery(
            "",
            {
              limit: request.limit,
              catalog: request.catalog,
              schema: request.schema,
              target: request.target,
              kind: request.kind,
            },
            onBatch,
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
