import { describe, expect, mock, test } from "bun:test";
import { createMcpIdentityHost } from "./identity.js";
import type { ToolResultLike } from "./protocol.js";

function host(answer: ToolResultLike | Error) {
  const callTool = mock(
    async (_name: string, _args: Record<string, unknown>) => {
      if (answer instanceof Error) throw answer;
      return answer;
    },
  );
  return { callTool, identity: createMcpIdentityHost(callTool) };
}

describe("createMcpIdentityHost", () => {
  test("search calls search_principals and keeps well-formed principals", async () => {
    const { callTool, identity } = host({
      structuredContent: {
        available: true,
        principals: [
          { kind: "user", name: "bob@example.com", displayName: "Bob" },
          { kind: "group", name: "data-eng" },
          { kind: "robot", name: "r2" },
          { kind: "user", name: "" },
          "junk",
        ],
      },
    });
    const found = await identity.search(" bo ", { kinds: ["user"], limit: 5 });
    expect(callTool.mock.calls[0]).toEqual([
      "search_principals",
      { query: "bo", kinds: ["user"], limit: 5 },
    ]);
    expect(found).toEqual([
      { kind: "user", name: "bob@example.com", displayName: "Bob" },
      { kind: "group", name: "data-eng" },
    ]);
  });

  test("a blank query doesn't call the server", async () => {
    const { callTool, identity } = host({ structuredContent: {} });
    expect(await identity.search("   ")).toEqual([]);
    expect(callTool).not.toHaveBeenCalled();
  });

  test("no directory (or a failed call) finds nothing", async () => {
    expect(
      await host({ structuredContent: { available: false } }).identity.search(
        "bo",
      ),
    ).toEqual([]);
    expect(
      await host({
        isError: true,
        structuredContent: { principals: [] },
      }).identity.search("bo"),
    ).toEqual([]);
  });

  test("an aborted search answers nothing", async () => {
    const { identity } = host({
      structuredContent: { principals: [{ kind: "user", name: "bob" }] },
    });
    const controller = new AbortController();
    const pending = identity.search("bo", { signal: controller.signal });
    controller.abort();
    expect(await pending).toEqual([]);
  });

  test("current asks whoami", async () => {
    const { callTool, identity } = host({
      structuredContent: {
        principal: { kind: "user", name: "me@example.com" },
      },
    });
    expect(await identity.current?.()).toEqual({
      kind: "user",
      name: "me@example.com",
    });
    expect(callTool.mock.calls[0]?.[0]).toBe("whoami");
    expect(
      await host({ structuredContent: {} }).identity.current?.(),
    ).toBeNull();
  });
});
