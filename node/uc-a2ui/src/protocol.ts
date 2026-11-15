// The contract between `uc mcp serve` (crates/cli/src/mcp) and this app:
// ids, tool names, the data model the grant surface binds to, and how A2UI
// messages ride in MCP tool results. Framework-free. The Rust side mirrors
// these constants in crates/cli/src/mcp/a2ui.rs; the fixture test
// (fixture.test.ts) checks a payload the server emitted against this app's
// catalogs.

import type { GrantSecurableType } from "@open-lakehouse/unity-catalog-client";
import { grantSubmission } from "@open-lakehouse/unity-catalog-ui/grants";

/** The A2UI protocol version the server speaks and the React renderer reads. */
export const A2UI_VERSION = "v0.9";

/** The catalog holding the Unity Catalog components (see catalog.tsx). */
export const UC_CATALOG_ID =
  "https://open-lakehouse.dev/a2ui/uc/v1/catalog.json";

/** The surface `open_grant_form` creates. */
export const GRANT_SURFACE_ID = "uc-grant-form";

/** MIME types of an embedded resource holding A2UI messages. A2UI's MCP guide
 *  names the first; some servers send the second. */
export const A2UI_MIME_TYPES: readonly string[] = [
  "application/a2ui+json",
  "application/json+a2ui",
];

/** The MCP tools the app calls (all `visibility: ["app"]` or shared). */
export const TOOLS = {
  applyGrants: "apply_grants",
  searchPrincipals: "search_principals",
  whoami: "whoami",
} as const;

/** The action the grant form's submit button fires. */
export const APPLY_GRANTS_EVENT = "apply_grants";

/** `/status` in the grant surface's data model. */
export interface GrantStatus {
  state: "idle" | "pending" | "ok" | "error";
  message?: string;
}

/** The parts of an MCP `CallToolResult` this app reads. */
export interface ToolResultLike {
  content?: ReadonlyArray<{
    type: string;
    resource?: { uri?: string; mimeType?: string; text?: string };
  }>;
  structuredContent?: unknown;
  isError?: boolean;
}

/**
 * The A2UI messages carried in `result`, in order: every embedded resource
 * with an A2UI MIME type holds a JSON array of messages (or one message).
 * Resources that don't parse are skipped, so a bad block can't hide the rest.
 */
export function a2uiMessages(result: ToolResultLike): unknown[] {
  const messages: unknown[] = [];
  for (const block of result.content ?? []) {
    const resource = block.type === "resource" ? block.resource : undefined;
    if (!resource?.text || !A2UI_MIME_TYPES.includes(resource.mimeType ?? "")) {
      continue;
    }
    try {
      const parsed: unknown = JSON.parse(resource.text);
      if (Array.isArray(parsed)) messages.push(...parsed);
      else messages.push(parsed);
    } catch {
      // Not JSON; ignore this block.
    }
  }
  return messages;
}

/** The arguments of the `apply_grants` tool. */
export interface ApplyGrantsArgs {
  securable_type: GrantSecurableType;
  full_name: string;
  changes: { principal: string; add: string[] }[];
}

const SECURABLE_TYPES: readonly GrantSecurableType[] = [
  "metastore",
  "catalog",
  "schema",
  "table",
  "function",
  "volume",
  "registered_model",
  "external_location",
  "credential",
];

function isSecurableType(value: unknown): value is GrantSecurableType {
  return SECURABLE_TYPES.includes(value as GrantSecurableType);
}

/**
 * The `apply_grants` arguments for a submit action's resolved context
 * (`securable_type`, `full_name`, `principal`, `privileges`), keeping only
 * the privileges that apply to the securable type. `null` when the context
 * doesn't name a securable, a principal and an applicable privilege.
 */
export function applyGrantsArgs(
  context: Record<string, unknown>,
): ApplyGrantsArgs | null {
  const { securable_type, full_name, principal, privileges } = context;
  if (!isSecurableType(securable_type)) return null;
  if (typeof full_name !== "string" || !full_name) return null;
  const grant = grantSubmission(securable_type, {
    principal: typeof principal === "string" ? principal : "",
    privileges: Array.isArray(privileges)
      ? privileges.filter((p): p is string => typeof p === "string")
      : [],
  });
  if (!grant.principal || grant.add.length === 0) return null;
  return { securable_type, full_name, changes: [grant] };
}
