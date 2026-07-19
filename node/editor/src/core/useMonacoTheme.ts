// Bridge the editor's Monaco theme to the app's ui-kit theme.
//
// ui-kit's `useTheme()` returns "dark" | "light" | "system" and toggles the
// `dark` class on <html>, but does NOT expose the resolved system value — so we
// resolve "system" here via `matchMedia` and keep Monaco's global theme in sync.
//
// Phase 1 maps to Monaco's built-in `vs` / `vs-dark`. Translating the shadcn CSS
// token palette into a custom `monaco.editor.defineTheme` is deferred (it means
// resolving ~40 token rules from runtime HSL variables — high effort, brittle);
// `vs`/`vs-dark` close the "hardcoded theme" gap correctly for now.

import { loader } from "@monaco-editor/react";
import { useTheme } from "@open-lakehouse/ui-kit";
import { useEffect, useState } from "react";

export type MonacoThemeId = "vs" | "vs-dark";

/** Resolve ui-kit's theme to a Monaco built-in theme id and keep Monaco synced.
 *  Returns the active theme id for callers that pass it to `<Editor theme>`. */
export function useMonacoTheme(): MonacoThemeId {
  const { theme } = useTheme();
  const [systemDark, setSystemDark] = useState(() =>
    typeof window === "undefined"
      ? false
      : window.matchMedia("(prefers-color-scheme: dark)").matches,
  );

  useEffect(() => {
    if (theme !== "system") return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => setSystemDark(media.matches);
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);

  const dark = theme === "dark" || (theme === "system" && systemDark);
  const monacoTheme: MonacoThemeId = dark ? "vs-dark" : "vs";

  useEffect(() => {
    // Global; safe to call repeatedly. Also re-themes editors already mounted.
    loader
      .init()
      .then((m) => m.editor.setTheme(monacoTheme))
      .catch(() => {
        // Loader not ready yet; the <Editor theme> prop covers the initial paint.
      });
  }, [monacoTheme]);

  return monacoTheme;
}
