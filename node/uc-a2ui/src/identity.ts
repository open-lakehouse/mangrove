// An `IdentityHost` over MCP tools. The MCP App runs in a sandboxed iframe
// with no network and no token, so it can't use `createScimIdentityHost`;
// instead `uc mcp serve` asks the server's SCIM API on the user's behalf
// (`search_principals`, `whoami`) and this host calls those tools. Mounted
// with `<IdentityProvider>`, it makes the grant form's principal field
// autocomplete exactly as it does in the web app. Framework-free.

import type {
  IdentityHost,
  PrincipalKind,
  PrincipalRef,
} from "@open-lakehouse/unity-catalog-client";
import { TOOLS, type ToolResultLike } from "./protocol.js";

/** Calls an MCP server tool, e.g. `app.callServerTool` from ext-apps. */
export type CallTool = (
  name: string,
  args: Record<string, unknown>,
) => Promise<ToolResultLike>;

const KINDS: readonly PrincipalKind[] = ["user", "group", "service_account"];

function toRef(value: unknown): PrincipalRef | null {
  if (!value || typeof value !== "object") return null;
  const { kind, name, displayName } = value as Record<string, unknown>;
  if (!KINDS.includes(kind as PrincipalKind)) return null;
  if (typeof name !== "string" || !name) return null;
  return typeof displayName === "string" && displayName
    ? { kind: kind as PrincipalKind, name, displayName }
    : { kind: kind as PrincipalKind, name };
}

/** `structuredContent` of a successful result, else `undefined`. */
function structured(
  result: ToolResultLike,
): Record<string, unknown> | undefined {
  if (result.isError) return undefined;
  const content = result.structuredContent;
  return content && typeof content === "object"
    ? (content as Record<string, unknown>)
    : undefined;
}

/**
 * An {@link IdentityHost} backed by the `search_principals` and `whoami`
 * tools. A server without a SCIM directory answers `{available: false}`;
 * search then finds nothing and the field takes principals as typed.
 */
export function createMcpIdentityHost(callTool: CallTool): IdentityHost {
  return {
    async search(query, options = {}) {
      const trimmed = query.trim();
      if (!trimmed) return [];
      const args: Record<string, unknown> = { query: trimmed };
      if (options.kinds) args.kinds = options.kinds;
      if (options.limit !== undefined) args.limit = options.limit;
      const result = await callTool(TOOLS.searchPrincipals, args);
      if (options.signal?.aborted) return [];
      const principals = structured(result)?.principals;
      if (!Array.isArray(principals)) return [];
      return principals.map(toRef).filter((r): r is PrincipalRef => !!r);
    },
    async current() {
      const result = await callTool(TOOLS.whoami, {});
      return toRef(structured(result)?.principal);
    },
  };
}
