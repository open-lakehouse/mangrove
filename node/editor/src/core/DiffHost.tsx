// A read-only Monaco diff of one file, before and after: e.g. what a change
// to an environment does to the files it renders. An added file has no
// `original`, a removed one no `modified`; both show against an empty side.

import { DiffEditor, type DiffOnMount } from "@monaco-editor/react";
import type * as Monaco from "monaco-editor";
import { useEffect, useRef } from "react";
import { monacoLanguageOfPath } from "./language.js";
import { ensureMonacoSetup } from "./monaco-setup.js";
import { READ_ONLY_OPTIONS } from "./ReadOnlyFile.js";
import { useMonacoTheme } from "./useMonacoTheme.js";

ensureMonacoSetup();

const DIFF_OPTIONS: Monaco.editor.IDiffEditorConstructionOptions = {
  ...READ_ONLY_OPTIONS,
  originalEditable: false,
  renderSideBySide: true,
  useInlineViewWhenSpaceIsLimited: true,
};

export interface DiffHostProps {
  /** The file's path: picks the language (see `monacoLanguageOfPath`). */
  path: string;
  /** The contents before; absent for an added file. */
  original?: string;
  /** The contents after; absent for a removed file. */
  modified?: string;
  /** Classes for the wrapper, which the editor fills (defaults to `h-full`). */
  className?: string;
}

/** One file's change, read-only, side by side (inline when narrow). */
export function DiffHost({
  path,
  original,
  modified,
  className,
}: DiffHostProps) {
  const theme = useMonacoTheme();
  const editorRef = useRef<Monaco.editor.IStandaloneDiffEditor | null>(null);
  const onMount: DiffOnMount = (editor) => {
    editorRef.current = editor;
  };
  // The diff widget must let go of its models before they are disposed, or
  // Monaco throws ("TextModel got disposed before DiffEditorWidget model got
  // reset"). The wrapper keeps them (`keepCurrent*Model`) and this disposes
  // them in the right order.
  useEffect(
    () => () => {
      const editor = editorRef.current;
      const models = editor?.getModel();
      editor?.setModel(null);
      models?.original.dispose();
      models?.modified.dispose();
    },
    [],
  );
  return (
    <div className={className ?? "h-full"} data-path={path}>
      <DiffEditor
        original={original ?? ""}
        modified={modified ?? ""}
        language={monacoLanguageOfPath(path)}
        theme={theme}
        options={DIFF_OPTIONS}
        onMount={onMount}
        keepCurrentOriginalModel
        keepCurrentModifiedModel
      />
    </div>
  );
}
