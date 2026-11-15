import { describe, expect, test } from "bun:test";
import { validatePayload } from "@a2ui/agent";
import { basicCatalog } from "@a2ui/web_core/v0_9/basic_catalog";
import fixture from "../fixtures/open_grant_form.json" with { type: "json" };
import { ucCatalog } from "./catalog.js";
import { A2UI_VERSION, GRANT_SURFACE_ID, UC_CATALOG_ID } from "./protocol.js";

// fixtures/open_grant_form.json is what `uc mcp serve`'s `open_grant_form`
// emits (crates/cli/src/mcp/a2ui.rs writes it and fails when it is stale), so
// this checks the server's payload against the catalogs the app renders with.
describe("open_grant_form payload", () => {
  test("is valid A2UI for the basic + Unity Catalog catalogs", () => {
    expect(() =>
      validatePayload([basicCatalog, ucCatalog], fixture),
    ).not.toThrow();
  });

  test("speaks the app's protocol constants", () => {
    const [create] = fixture as [
      {
        version: string;
        createSurface: { surfaceId: string; catalogId: string };
      },
    ];
    expect(create.version).toBe(A2UI_VERSION);
    expect(create.createSurface).toEqual({
      surfaceId: GRANT_SURFACE_ID,
      catalogId: UC_CATALOG_ID,
    });
  });

  test("a component the catalog doesn't define is rejected", () => {
    const bad = structuredClone(fixture) as unknown as [
      unknown,
      { updateComponents: { components: { component: string }[] } },
    ];
    for (const c of bad[1].updateComponents.components) {
      c.component = "UcRevokeForm";
    }
    expect(() => validatePayload([basicCatalog, ucCatalog], bad)).toThrow();
  });
});
