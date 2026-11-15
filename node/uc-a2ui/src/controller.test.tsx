import "../../unity-catalog-ui/src/testing/dom.js";

import { afterEach, describe, expect, mock, test } from "bun:test";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import fixture from "../fixtures/open_grant_form.json" with { type: "json" };
import { createGrantController } from "./controller.js";
import {
  A2UI_VERSION,
  GRANT_SURFACE_ID,
  type ToolResultLike,
} from "./protocol.js";
import { Surfaces } from "./Surfaces.js";

afterEach(cleanup);

const a2ui = (messages: unknown[]): ToolResultLike => ({
  content: [
    {
      type: "resource",
      resource: {
        uri: "a2ui://uc/grant-form",
        mimeType: "application/a2ui+json",
        text: JSON.stringify(messages),
      },
    },
  ],
});

const update = (value: unknown, path = "/status") => ({
  version: A2UI_VERSION,
  updateDataModel: { surfaceId: GRANT_SURFACE_ID, path, value },
});

function setup(answer: (args: Record<string, unknown>) => ToolResultLike) {
  const callTool = mock(async (_name: string, args: Record<string, unknown>) =>
    answer(args),
  );
  const updateModelContext = mock(async (_text: string) => ({}));
  const controller = createGrantController({ callTool, updateModelContext });
  render(<Surfaces processor={controller.processor} empty={<p>empty</p>} />);
  act(() => controller.applyToolResult(a2ui(fixture)));
  return { callTool, updateModelContext, controller };
}

const principalField = () =>
  screen.getByLabelText("Principal") as HTMLInputElement;

describe("grant surface", () => {
  test("renders the prefilled form from open_grant_form's payload", async () => {
    setup(() => ({}));
    expect(await screen.findByText("main.sales.orders")).toBeTruthy();
    expect(principalField().value).toBe("bob");
    expect(
      (screen.getByRole("checkbox", { name: "SELECT" }) as HTMLInputElement)
        .checked,
    ).toBe(true);
    // The ambiguous hint's matches, and the current grants.
    expect(
      screen.getByRole("button", { name: "Bob Builder (bob@example.com)" }),
    ).toBeTruthy();
    expect(screen.getByText(/MODIFY, SELECT/)).toBeTruthy();
  });

  test("submit calls apply_grants with the edited grant and shows the answer", async () => {
    const { callTool, updateModelContext } = setup(() => ({
      content: [
        { type: "text", text: "Granted SELECT, MODIFY to bob@example.com" },
        ...(a2ui([update({ state: "ok", message: "Granted SELECT, MODIFY." })])
          .content ?? []),
      ],
    }));
    const user = userEvent.setup();
    await screen.findByText("main.sales.orders");
    await user.click(
      screen.getByRole("button", { name: "Bob Builder (bob@example.com)" }),
    );
    await user.click(screen.getByRole("checkbox", { name: "MODIFY" }));
    await user.click(screen.getByRole("button", { name: "Grant" }));

    await waitFor(() => expect(callTool).toHaveBeenCalledTimes(1));
    expect(callTool.mock.calls[0]).toEqual([
      "apply_grants",
      {
        securable_type: "table",
        full_name: "main.sales.orders",
        changes: [{ principal: "bob@example.com", add: ["SELECT", "MODIFY"] }],
      },
    ]);
    expect((await screen.findByRole("status")).textContent).toBe(
      "Granted SELECT, MODIFY.",
    );
    await waitFor(() =>
      expect(updateModelContext.mock.calls[0]?.[0]).toBe(
        "Granted SELECT, MODIFY to bob@example.com",
      ),
    );
  });

  test("a failed grant shows the server's error", async () => {
    setup(() => ({
      isError: true,
      content: [{ type: "text", text: "PERMISSION_DENIED: not an owner" }],
    }));
    const user = userEvent.setup();
    await screen.findByText("main.sales.orders");
    await user.click(screen.getByRole("button", { name: "Grant" }));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "PERMISSION_DENIED: not an owner",
    );
  });

  test("a transport failure shows as an error too", async () => {
    setup(() => {
      throw new Error("host went away");
    });
    const user = userEvent.setup();
    await screen.findByText("main.sales.orders");
    await user.click(screen.getByRole("button", { name: "Grant" }));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "host went away",
    );
  });

  test("privileges that don't apply to the securable are called out", async () => {
    const { controller } = setup(() => ({}));
    await screen.findByText("main.sales.orders");
    act(() =>
      controller.applyToolResult(
        a2ui([update(["SELECT", "CREATE_CATALOG"], "/privileges")]),
      ),
    );
    expect(
      await screen.findByText("Not grantable on a table: CREATE CATALOG"),
    ).toBeTruthy();
  });

  test("reopening the form replaces the surface", async () => {
    const { controller } = setup(() => ({}));
    await screen.findByText("main.sales.orders");
    const reopened = structuredClone(fixture) as typeof fixture;
    (
      reopened[2] as {
        updateDataModel: { value: { securable: { fullName: string } } };
      }
    ).updateDataModel.value.securable.fullName = "main.sales.returns";
    act(() => controller.applyToolResult(a2ui(reopened)));
    expect(await screen.findByText("main.sales.returns")).toBeTruthy();
    expect(screen.queryByText("main.sales.orders")).toBeNull();
    expect(controller.processor.model.surfacesMap.size).toBe(1);
  });
});
