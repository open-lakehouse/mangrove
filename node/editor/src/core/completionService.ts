// Catalog-aware SQL completion, built as a monaco-sql-languages `CompletionService`.
//
// monaco-sql-languages runs the ANTLR PostgreSQL parser in its web worker and
// hands the parse result — `suggestions` (expected-entity syntax context +
// valid keywords) and `entities` (referenced tables/columns) — to this callback,
// which runs on the MAIN thread. That split is exactly what we want: the heavy
// parse is off-thread, while catalog lookups (async REST fetches through the
// pluggable `CatalogProvider`) are natural here on the main thread.
//
// We source real catalog/schema/table/column names from the pluggable provider
// (see catalogProvider.ts) and narrow by the dotted path under the cursor. When
// no provider is registered the provider returns empty lists, so completion
// silently degrades to the keywords the worker already produced.

// Import the parser enum/types from dt-sql-parser directly (monaco-sql-languages
// re-exports them). The `monaco-sql-languages` barrel pulls in the monaco runtime
// (which touches `window` at import), so importing the enum from the parser keeps
// this pure-logic module free of any monaco runtime import — testable in Node.
import { EntityContextType, type Suggestions } from "dt-sql-parser";
import type * as Monaco from "monaco-editor";
import type { CompletionService, ICompletionItem } from "monaco-sql-languages";
import { type CatalogColumn, getCatalogProvider } from "./catalogProvider";

// monaco.languages.CompletionItemKind values, inlined so this module carries NO
// runtime import of `monaco-editor` (which touches `window` at import time and
// would drag the whole editor into a non-DOM context — e.g. bun unit tests).
// These are stable monaco enum constants.
const Kind = {
  Field: 3,
  Module: 8,
  Folder: 18,
} as const;

// Catalog metadata changes infrequently; cache lookups for a minute so typing
// doesn't hammer the source. (Invalidation isn't critical for completion.)
const TTL_MS = 60_000;

interface Cached<T> {
  at: number;
  value: Promise<T>;
}
const cache = new Map<string, Cached<unknown>>();

function memo<T>(key: string, fn: () => Promise<T>): Promise<T> {
  const hit = cache.get(key) as Cached<T> | undefined;
  if (hit && performance.now() - hit.at < TTL_MS) return hit.value;
  const value = fn().catch((err) => {
    cache.delete(key);
    throw err;
  });
  cache.set(key, { at: performance.now(), value });
  return value;
}

const listCatalogs = () =>
  memo("catalogs", () => getCatalogProvider().catalogs());
const listSchemas = (catalog: string) =>
  memo(`schemas:${catalog}`, () => getCatalogProvider().schemas(catalog));
const listTables = (catalog: string, schema: string) =>
  memo(`tables:${catalog}.${schema}`, () =>
    getCatalogProvider().tables(catalog, schema),
  );
const listColumns = (fullTableName: string) =>
  memo(`columns:${fullTableName}`, () =>
    getCatalogProvider().columns(fullTableName),
  );

/**
 * The dotted-name prefix immediately before the cursor, derived from the line
 * text — NOT from the parser. dt-sql-parser returns an empty `syntax` once a
 * trailing dot is typed (`main.|`), so we can't get the path from `wordRanges`;
 * we read it off the text instead. Returns the segments BEFORE the partial word
 * being typed: `main.|` → ["main"]; `main.sa|` → ["main"]; `main.sales.|` →
 * ["main","sales"]; `foo|` (no dot) → []. Returns null if the cursor isn't in a
 * dotted identifier path at all.
 */
function dottedPrefix(lineUpToCursor: string): string[] | null {
  // Grab the trailing run of `word.word.word` (with the optional partial word).
  const m = lineUpToCursor.match(/([A-Za-z_][\w]*\.)+[\w]*$/);
  if (!m) return null;
  const segments = m[0].split(".");
  // The last segment is the partial word being completed; drop it.
  return segments.slice(0, -1).map((s) => s.replace(/[`"]/g, ""));
}

/** Fully-qualified `catalog.schema.table` names referenced in the statement. */
function tablesInStatement(suggestions: Suggestions): string[] {
  const tables = new Set<string>();
  for (const s of suggestions.syntax) {
    if (s.syntaxContextType === EntityContextType.TABLE) {
      const parts = s.wordRanges.map((w) => w.text.replace(/[.`"]/g, ""));
      if (parts.length === 3) tables.add(parts.join("."));
    }
  }
  return [...tables];
}

interface RawItem {
  label: string;
  kind: number;
  detail: string;
  sort: string;
}

/**
 * Build completion items. When the cursor is inside a dotted path (`prefix` has
 * ≥1 segment), we narrow purely by prefix length — this is the reliable path,
 * since the parser yields no `syntax` after a trailing dot. Otherwise we use the
 * parser's `syntaxContextType` to decide whether catalogs, columns, etc. are
 * expected.
 */
async function buildItems(
  suggestions: Suggestions | null,
  prefix: string[],
): Promise<RawItem[]> {
  const out: RawItem[] = [];
  const seen = new Set<string>();
  const push = (label: string, kind: number, detail: string, sort: string) => {
    const key = `${kind}:${label}`;
    if (!seen.has(key)) {
      seen.add(key);
      out.push({ label, kind, detail, sort });
    }
  };

  try {
    if (prefix.length >= 3) {
      // catalog.schema.table. → columns of that table.
      const table = prefix.slice(0, 3).join(".");
      for (const col of await listColumns(table))
        push(
          col.name,
          Kind.Field,
          col.type ? `${col.type} · ${table}` : table,
          "1",
        );
    } else if (prefix.length === 2) {
      // catalog.schema. → tables.
      for (const t of await listTables(prefix[0], prefix[1]))
        push(t, Kind.Field, `table in ${prefix[0]}.${prefix[1]}`, "1");
    } else if (prefix.length === 1) {
      // catalog. → schemas.
      for (const s of await listSchemas(prefix[0]))
        push(s, Kind.Folder, `schema in ${prefix[0]}`, "1");
    } else {
      // No dotted prefix: use the parser's expected-entity context.
      const types = new Set(
        suggestions?.syntax.map((s) => s.syntaxContextType) ?? [],
      );
      if (
        types.has(EntityContextType.TABLE) ||
        types.has(EntityContextType.VIEW) ||
        types.has(EntityContextType.CATALOG) ||
        types.has(EntityContextType.DATABASE)
      ) {
        for (const c of await listCatalogs())
          push(c, Kind.Module, "catalog", "2");
      }
      if (suggestions && types.has(EntityContextType.COLUMN)) {
        for (const table of tablesInStatement(suggestions)) {
          for (const col of await listColumns(table))
            push(
              col.name,
              Kind.Field,
              col.type ? `${col.type} · ${table}` : table,
              "1",
            );
        }
      }
    }
  } catch {
    // A failed catalog lookup must not break keyword completion.
  }

  return out;
}

/**
 * The catalog-aware `CompletionService` to hand to `setupLanguageFeatures`. The
 * worker supplies `suggestions` (its parse of expected entities + keywords); we
 * add real catalog names narrowed by the dotted path under the cursor.
 *
 * Note we return ONLY the catalog entities we source; the package merges these
 * with its own keyword suggestions (built from `suggestions.keywords`), so we do
 * not re-emit keywords here — that keeps keyword handling in the package and our
 * job focused on live catalog names.
 */
export const catalogCompletionService: CompletionService = async (
  model,
  position,
  _context,
  suggestions,
) => {
  // The dotted prefix comes from the line text (the parser drops it after a
  // trailing dot), and decides catalog→schema→table→column narrowing.
  const lineUpToCursor = model.getValueInRange({
    startLineNumber: position.lineNumber,
    startColumn: 1,
    endLineNumber: position.lineNumber,
    endColumn: position.column,
  });
  const prefix = dottedPrefix(lineUpToCursor) ?? [];

  // With nothing parsed and no dotted prefix, there's nothing catalog-shaped to
  // offer (the package still contributes its keyword items).
  if (!suggestions && prefix.length === 0) return [];

  // Replace the word under the cursor (so completing a partial name works).
  const word = model.getWordUntilPosition(position);
  const range: Monaco.IRange = {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  };

  const items = await buildItems(suggestions, prefix);
  const completions: ICompletionItem[] = items.map((it) => ({
    label: it.label,
    kind: it.kind,
    detail: it.detail,
    insertText: it.label,
    sortText: it.sort + it.label,
    range,
  }));
  return completions;
};

/** Exported for unit testing the narrowing logic without a live editor. */
export const __test = { dottedPrefix, tablesInStatement, buildItems };
export type { CatalogColumn };
