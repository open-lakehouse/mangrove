import { describe, expect, test } from "bun:test";
import { a2uiMessages, applyGrantsArgs } from "./protocol.js";

const resource = (mimeType: string, text: string) => ({
  type: "resource",
  resource: { uri: "a2ui://uc/grant-form", mimeType, text },
});

describe("a2uiMessages", () => {
  test("flattens A2UI resources in order and skips the rest", () => {
    const messages = a2uiMessages({
      content: [
        { type: "text", text: "Opened a grant form" } as never,
        resource("application/a2ui+json", JSON.stringify([{ a: 1 }, { b: 2 }])),
        resource("text/html", "<p>not a2ui</p>"),
        resource("application/json+a2ui", JSON.stringify({ c: 3 })),
        resource("application/a2ui+json", "{not json"),
      ],
    });
    expect(messages).toEqual([{ a: 1 }, { b: 2 }, { c: 3 }]);
  });

  test("an empty result holds no messages", () => {
    expect(a2uiMessages({})).toEqual([]);
  });
});

describe("applyGrantsArgs", () => {
  test("keeps the privileges that apply, in menu order", () => {
    expect(
      applyGrantsArgs({
        securable_type: "schema",
        full_name: "main.sales",
        principal: " bob@example.com ",
        privileges: ["SELECT", "CREATE_CATALOG", "USE_SCHEMA", 7],
      }),
    ).toEqual({
      securable_type: "schema",
      full_name: "main.sales",
      changes: [
        { principal: "bob@example.com", add: ["USE_SCHEMA", "SELECT"] },
      ],
    });
  });

  test.each([
    ["an unknown securable type", { securable_type: "share" }],
    ["no full name", { full_name: "" }],
    ["no principal", { principal: "  " }],
    ["no applicable privilege", { privileges: ["CREATE_CATALOG"] }],
    ["privileges that aren't a list", { privileges: "SELECT" }],
  ])("is null for %s", (_, override) => {
    expect(
      applyGrantsArgs({
        securable_type: "schema",
        full_name: "main.sales",
        principal: "bob",
        privileges: ["SELECT"],
        ...override,
      }),
    ).toBeNull();
  });
});
