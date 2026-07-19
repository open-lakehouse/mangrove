// Centralized, run-once Monaco bootstrap.
//
// Three things happen before any editor mounts, exactly once for the whole app
// (not per editor, not per tab):
//
//  1. Point `@monaco-editor/react`'s loader at the *bundled* `monaco-editor`
//     package rather than its default CDN download — the CDN path doesn't work
//     offline, desyncs from the `monaco-sql-languages` build (pinned to this
//     monaco version), and can't be locked to our tested version.
//
//  2. Install `self.MonacoEnvironment.getWorker`, returning the right worker per
//     language `label`: the `monaco-sql-languages` pgsql worker for `pgsql`,
//     otherwise the base editor worker (diff/links/etc.). Both are wired via
//     Vite's `?worker` import so each bundles as its own worker chunk.
//
//  3. Register the pgsql language contribution and enable its features:
//     diagnostics (validation, in the worker) and completionItems with our
//     catalog-aware `CompletionService`. The parser runs in the worker; our
//     completion callback runs on the main thread with the worker's parse
//     context (see completionService.ts).
//
// This uses `monaco-sql-languages`' OWN worker path (no hand-wired handshake) —
// it works on the tested monaco-editor@0.52.2 + monaco-sql-languages@1.1 pair.
// See README.md for the version constraint and consuming-app build steps.
//
// Import this module for its side effects once, early — `ensureMonacoSetup()`
// is idempotent and StrictMode-safe, so calling it from a component mount is fine.

import { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
// Base Monaco editor worker (the editorWorkerService — diff/links/etc.). Wired
// via Vite's `?worker` import, which bundles it as a separate worker chunk.
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
// The monaco-sql-languages pgsql worker (ANTLR PostgreSQL parser: diagnostics +
// the completion parse context). `?worker` bundles it into its own chunk.
import PgSqlWorker from "monaco-sql-languages/esm/languages/pgsql/pgsql.worker?worker";
// Register the pgsql language contribution (tokenizer + language features). This
// calls monaco's `registerLanguage` at import time; it does not create workers.
import "monaco-sql-languages/esm/languages/pgsql/pgsql.contribution";
import { LanguageIdEnum, setupLanguageFeatures } from "monaco-sql-languages";
import { catalogCompletionService } from "./completionService";

let done = false;

/**
 * Idempotently configure the Monaco loader + workers + SQL features. Safe to
 * call repeatedly (e.g. from a component mount under React StrictMode); only the
 * first call has any effect.
 */
export function ensureMonacoSetup(): void {
  if (done) return;
  done = true;

  // Per-language worker routing. `label` is the language id; the pgsql worker
  // MUST be returned for `LanguageIdEnum.PG` or completion silently won't parse.
  self.MonacoEnvironment = {
    getWorker(_workerId: string, label: string) {
      if (label === LanguageIdEnum.PG) {
        return new PgSqlWorker();
      }
      return new EditorWorker();
    },
  };

  // Use the bundled monaco rather than the CDN default.
  loader.config({ monaco });

  // Enable diagnostics (validation, in the worker) and completion. Completion
  // runs the parser in the worker and hands the parse context to our main-thread
  // catalog-aware `completionService`; `.` and space are the trigger characters.
  setupLanguageFeatures(LanguageIdEnum.PG, {
    diagnostics: true,
    completionItems: {
      enable: true,
      triggerCharacters: [" ", "."],
      completionService: catalogCompletionService,
    },
  });
}
