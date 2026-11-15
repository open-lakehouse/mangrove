// The app's logic, apart from React and the MCP Apps transport: a
// `MessageProcessor` over the basic and Unity Catalog catalogs, fed from tool
// results, whose grant form's submit calls `apply_grants` and feeds the answer
// back into the surface.

import type { ActionPayload } from "@a2ui/web_core";
import { MessageProcessor } from "@a2ui/web_core/v0_9";
import { basicCatalog } from "@a2ui/web_core/v0_9/basic_catalog";
import { ucCatalog } from "./catalog.js";
import type { CallTool } from "./identity.js";
import {
  A2UI_VERSION,
  APPLY_GRANTS_EVENT,
  a2uiMessages,
  applyGrantsArgs,
  type GrantStatus,
  TOOLS,
  type ToolResultLike,
} from "./protocol.js";

export interface GrantControllerDeps {
  callTool: CallTool;
  /** Tells the agent what happened, without starting a turn
   *  (`app.updateModelContext`). Optional: not every host supports it. */
  updateModelContext?: (text: string) => Promise<unknown>;
}

/** The text blocks of `result`, joined. */
export function resultText(result: ToolResultLike): string {
  return (result.content ?? [])
    .map((block) =>
      block.type === "text" && "text" in block && typeof block.text === "string"
        ? block.text
        : "",
    )
    .filter(Boolean)
    .join("\n");
}

function createSurfaceId(message: unknown): string | undefined {
  if (!message || typeof message !== "object") return undefined;
  const create = (message as { createSurface?: { surfaceId?: unknown } })
    .createSurface;
  return typeof create?.surfaceId === "string" ? create.surfaceId : undefined;
}

export function createGrantController(deps: GrantControllerDeps) {
  // `processor` is assigned before any action can fire.
  let processor: MessageProcessor;

  function setStatus(surfaceId: string, status: GrantStatus) {
    processor.model.getSurface(surfaceId)?.dataModel.set("/status", status);
  }

  /** Apply A2UI messages; a `createSurface` for a live surface replaces it
   *  (the agent reopened the form). */
  function apply(messages: unknown[]) {
    for (const message of messages) {
      const id = createSurfaceId(message);
      if (id && processor.model.getSurface(id)) {
        processor.processMessages([
          { version: A2UI_VERSION, deleteSurface: { surfaceId: id } },
        ]);
      }
      processor.processMessages([message] as never);
    }
  }

  async function onAction(action: ActionPayload) {
    if (action.name !== APPLY_GRANTS_EVENT) return;
    const args = applyGrantsArgs(action.context);
    if (!args) {
      setStatus(action.surfaceId, {
        state: "error",
        message: "Pick a principal and at least one privilege.",
      });
      return;
    }
    setStatus(action.surfaceId, { state: "pending" });
    let result: ToolResultLike;
    try {
      result = await deps.callTool(TOOLS.applyGrants, { ...args });
    } catch (error) {
      setStatus(action.surfaceId, {
        state: "error",
        message: error instanceof Error ? error.message : String(error),
      });
      return;
    }
    const messages = a2uiMessages(result);
    const text = resultText(result);
    if (messages.length > 0) apply(messages);
    else {
      setStatus(
        action.surfaceId,
        result.isError
          ? { state: "error", message: text || "The grant failed." }
          : { state: "ok", message: text || undefined },
      );
    }
    if (text && deps.updateModelContext) {
      // Best effort: the grant already happened either way.
      await deps.updateModelContext(text).catch(() => {});
    }
  }

  processor = new MessageProcessor([basicCatalog, ucCatalog], onAction);

  return {
    processor,
    /** Render what a tool result carries (the host's `ontoolresult`). */
    applyToolResult(result: ToolResultLike) {
      apply(a2uiMessages(result));
    },
  };
}

export type GrantController = ReturnType<typeof createGrantController>;
