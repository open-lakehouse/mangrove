// The Unity Catalog A2UI catalog and the pieces of the MCP App that hosts it,
// for embedders that render agent surfaces themselves (e.g. a chat pane in the
// lakehouse app). The MCP App entry is main.tsx.

export { UcGrantForm, UcGrantFormApi, ucCatalog } from "./catalog.js";
export {
  createGrantController,
  type GrantController,
  type GrantControllerDeps,
} from "./controller.js";
export { type CallTool, createMcpIdentityHost } from "./identity.js";
export * from "./protocol.js";
export { Surfaces } from "./Surfaces.js";
