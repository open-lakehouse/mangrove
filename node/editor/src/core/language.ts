// Path → editor language classification.
//
// One source of truth for "what kind of file is this", used by the file tree
// (icon), the tab (label/behavior), and MonacoHost (the model's language id).
// `sql` maps to the `pgsql` Monaco language registered by monaco-sql-languages
// (the query engine speaks PostgreSQL); markdown gets a preview; everything else
// (including `.py`) is plain text.

export type EditorLanguage = "sql" | "markdown" | "plaintext";

/** Monaco language id for a given EditorLanguage. */
export const MONACO_LANGUAGE_ID: Record<EditorLanguage, string> = {
  sql: "pgsql",
  markdown: "markdown",
  plaintext: "plaintext",
};

const BY_EXTENSION: Record<string, EditorLanguage> = {
  sql: "sql",
  md: "markdown",
  markdown: "markdown",
};

/** The lowercased extension (without the dot), or "" if none. */
export function extensionOf(path: string): string {
  const name = path.replace(/\/+$/, "").split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

/** Classify a path into an EditorLanguage (plaintext for unknown types). */
export function languageOf(path: string): EditorLanguage {
  const ext = extensionOf(path);
  return BY_EXTENSION[ext] ?? "plaintext";
}

/** Monaco language ids for read-only viewing (`ReadOnlyFile`, `DiffHost`), by
 *  extension. Broader than {@link languageOf}, which only classifies what the
 *  session can edit. Monaco has no TOML, so it and dotenv files read as INI. SQL
 *  is Monaco's own `sql` (Monarch highlighting only), not `pgsql`, so viewing
 *  generated SQL starts no worker and shows no diagnostics. */
const VIEW_BY_EXTENSION: Record<string, string> = {
  yaml: "yaml",
  yml: "yaml",
  json: "json",
  toml: "ini",
  ini: "ini",
  env: "ini",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  dockerfile: "dockerfile",
  sql: "sql",
  md: "markdown",
  markdown: "markdown",
};

/** The Monaco language id to view `path` in (`plaintext` for unknown types). */
export function monacoLanguageOfPath(path: string): string {
  const name = path.replace(/\/+$/, "").split("/").pop() ?? "";
  if (name === "Dockerfile" || name.startsWith("Dockerfile."))
    return "dockerfile";
  if (name === ".env" || name.startsWith(".env.")) return "ini";
  return VIEW_BY_EXTENSION[extensionOf(path)] ?? "plaintext";
}
