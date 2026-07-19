import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The Unity Catalog REST API and this SPA are served on one origin by the
// `uc-server` Rust binary (tower-http ServeDir serves the built `dist/` as
// `web/`). In dev, proxy the API path prefix to a locally-running UC server so
// the browser talks to a single origin.
const API_URL = process.env.UC_API_URL ?? "http://localhost:8080";

// The in-browser wasm query engine is OPT-IN per build: its worker imports the
// gitignored wasm-bindgen artifact under crates/query-wasm/pkg/ (produced by
// `just build-query-wasm`), which default builds must never try to resolve. So
// unless VITE_ENABLE_WASM_QUERY=true, alias the whole package to its committed
// no-op stub — `registerWasmPreview` then registers nothing and the preview UI
// stays dark (see @open-lakehouse/query-wasm).
const WASM_QUERY_ENABLED = process.env.VITE_ENABLE_WASM_QUERY === "true";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  // Emit relative asset URLs (`./assets/...`) so the bundle works under any
  // server base path without a rebuild — the server may mount it anywhere.
  base: "./",
  resolve: {
    // The shared UI packages are separate workspace packages; force a single
    // copy of React and the TanStack context/singleton libs so hooks and
    // query/router context work at runtime (duplicates break them).
    dedupe: [
      "react",
      "react-dom",
      "@tanstack/react-query",
      "@tanstack/react-router",
    ],
    alias: WASM_QUERY_ENABLED
      ? {
          // The worker imports the wasm-bindgen artifact via this bare name
          // (see @open-lakehouse/query-wasm src/pkg.d.ts).
          "query-wasm-pkg": path.resolve(
            __dirname,
            "../../crates/query-wasm/pkg/query_wasm.js",
          ),
        }
      : {
          // Default builds ship no wasm: alias both wasm engines to their no-op
          // stubs so neither tries to resolve the gitignored wasm-bindgen artifact
          // under crates/query-wasm/pkg/. The query preview + volume Files tab then
          // fall back to their dev stub runners (see main.tsx).
          "@open-lakehouse/query-wasm": "@open-lakehouse/query-wasm/stub",
          "@open-lakehouse/files-wasm": "@open-lakehouse/files-wasm/stub",
        },
  },
  server: {
    port: 3003,
    fs: {
      // Allow serving the sibling workspace-package sources (consumed directly
      // from their `src/` via the package `exports` map) — and, one level up,
      // the wasm engine artifact under crates/query-wasm/pkg/ in wasm builds.
      allow: [path.resolve(__dirname, "../..")],
    },
    proxy: {
      // Unity Catalog REST API (served same-origin as this SPA in production).
      "/api": {
        target: API_URL,
        changeOrigin: true,
      },
    },
  },
});
