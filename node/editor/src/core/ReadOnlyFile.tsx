// A read-only Monaco view of one file: a host's file browser or preview (e.g.
// the files an environment renders), not an editing surface. Unlike
// `MonacoHost` it owns no session model: the editor keeps one model of its
// own whose value and language follow the props, so two viewers of the same
// path (say, in two environments) never share contents.

import Editor from "@monaco-editor/react";
import type * as Monaco from "monaco-editor";
import { monacoLanguageOfPath } from "./language.js";
import { ensureMonacoSetup } from "./monaco-setup.js";
import { useMonacoTheme } from "./useMonacoTheme.js";

ensureMonacoSetup();

/** The options every read-only view shares. */
export const READ_ONLY_OPTIONS: Monaco.editor.IStandaloneEditorConstructionOptions =
  {
    readOnly: true,
    domReadOnly: true,
    minimap: { enabled: false },
    fontSize: 13,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    renderLineHighlight: "none",
  };

export interface ReadOnlyFileProps {
  /** The file's path: picks the language (see `monacoLanguageOfPath`). */
  path: string;
  /** The file's contents. */
  value: string;
  /** Classes for the wrapper, which the editor fills (defaults to `h-full`). */
  className?: string;
}

/** One file, read-only, highlighted by its extension, in the app's theme. */
export function ReadOnlyFile({ path, value, className }: ReadOnlyFileProps) {
  const theme = useMonacoTheme();
  return (
    <div className={className ?? "h-full"} data-path={path}>
      <Editor
        value={value}
        language={monacoLanguageOfPath(path)}
        theme={theme}
        options={READ_ONLY_OPTIONS}
      />
    </div>
  );
}
