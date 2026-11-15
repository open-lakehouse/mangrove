import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defaultClientConditions, defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

// Builds the MCP App `uc mcp serve` serves as `ui://uc/grants`: one HTML file
// with every script and style inlined. Hosts load it as an iframe `srcdoc`
// with no base URL, so it can't fetch chunks or assets.
export default defineConfig({
  plugins: [react(), tailwindcss(), viteSingleFile()],
  resolve: {
    // Build the workspace packages from source, like node/app does.
    conditions: ["source", ...defaultClientConditions],
    dedupe: ["react", "react-dom", "@tanstack/react-query"],
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // One inlined chunk by design.
    chunkSizeWarningLimit: 4096,
  },
});
