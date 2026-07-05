// Runner composition: try a primary runner, transparently re-run on a fallback
// when the primary fails BEFORE producing any chunk.
//
// This is the wasm→host composition point from WASM_QUERY_PREVIEW.md Phase B:
// the in-browser engine throws typed errors (an `Error` whose `code` property
// is "UNSUPPORTED" for tables outside its envelope, "NETWORK" for CORS-blocked
// direct storage fetches) and the composed runner retries the same request on
// the fallback — typically a host's server-backed runner. Once the primary has
// yielded a chunk, its errors propagate: rows may already be rendered, and
// silently restarting on another engine could produce a torn result.

import type { QueryChunk, QueryRunner } from "./runner";

/**
 * Compose two runners: `primary` first; on any failure before the first chunk
 * (narrow it via `opts.shouldFallBack`), the same request re-runs on
 * `fallback`. Aborts never fall back.
 */
export function createFallbackQueryRunner(
  primary: QueryRunner,
  fallback: QueryRunner,
  opts: {
    /** Decide per error whether to fall back (default: always). */
    shouldFallBack?: (error: unknown) => boolean;
    /** Observability hook: called when a fallback happens. */
    onFallback?: (error: unknown) => void;
  } = {},
): QueryRunner {
  return (req, runnerOpts) => ({
    async *[Symbol.asyncIterator](): AsyncIterator<QueryChunk> {
      let yielded = false;
      try {
        for await (const chunk of primary(req, runnerOpts)) {
          yielded = true;
          yield chunk;
        }
        return;
      } catch (error) {
        const fallBack =
          !yielded &&
          !runnerOpts.signal.aborted &&
          (opts.shouldFallBack?.(error) ?? true);
        if (!fallBack) throw error;
        opts.onFallback?.(error);
      }
      yield* fallback(req, runnerOpts);
    },
  });
}
