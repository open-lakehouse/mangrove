import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The Unity Catalog REST API and this SPA are served on one origin by the
// `uc-server` Rust binary (tower-http ServeDir serves the built `dist/` as
// `web/`). In dev, proxy the API path prefix to a locally-running UC server so
// the browser talks to a single origin.
const API_URL = process.env.UC_API_URL ?? "http://localhost:8080";

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
  },
  server: {
    port: 3003,
    fs: {
      // Allow serving the sibling workspace-package sources (consumed directly
      // from their `src/` via the package `exports` map).
      allow: [path.resolve(__dirname, "..")],
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
