// The MCP App `uc mcp serve` serves as `ui://uc/grants`. The host renders it
// next to an `open_grant_form` result and hands it that result; the app draws
// the A2UI surface it carries and sends the user's grant back through
// `apply_grants`. It has no network and no token: every Unity Catalog call is
// an MCP tool call that `uc` makes as the user.

import "./styles.css";

import {
  App,
  applyDocumentTheme,
  applyHostStyleVariables,
} from "@modelcontextprotocol/ext-apps";
import { IdentityProvider } from "@open-lakehouse/unity-catalog-client/react";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createGrantController } from "./controller.js";
import { type CallTool, createMcpIdentityHost } from "./identity.js";
import { Surfaces } from "./Surfaces.js";

const app = new App({ name: "uc-grants", version: "0.0.0" });

const callTool: CallTool = (name, args) =>
  app.callServerTool({ name, arguments: args });

const controller = createGrantController({
  callTool,
  updateModelContext: (text) =>
    app.updateModelContext({ content: [{ type: "text", text }] }),
});
const identity = createMcpIdentityHost(callTool);

function applyTheme() {
  const context = app.getHostContext();
  if (context?.theme) {
    applyDocumentTheme(context.theme);
    document.documentElement.classList.toggle("dark", context.theme === "dark");
  }
  if (context?.styles?.variables) {
    applyHostStyleVariables(context.styles.variables);
  }
}

// Handlers go in before `connect`, so the first result isn't missed.
app.ontoolresult = (result) => controller.applyToolResult(result);
app.onhostcontextchanged = () => applyTheme();

const root = document.getElementById("root");
if (!root) throw new Error("no #root");
createRoot(root).render(
  <StrictMode>
    <IdentityProvider identity={identity}>
      <main className="p-4">
        <Surfaces
          processor={controller.processor}
          empty={
            <p className="text-sm text-muted-foreground">Loading the form…</p>
          }
        />
      </main>
    </IdentityProvider>
  </StrictMode>,
);

app.connect().then(applyTheme, (error: unknown) => {
  console.error("MCP Apps handshake failed", error);
});
