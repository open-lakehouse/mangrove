import "../testing/dom.js";

import { afterEach, describe, expect, test } from "bun:test";
import type { IdentityHost } from "@open-lakehouse/unity-catalog-client";
import { IdentityProvider } from "@open-lakehouse/unity-catalog-client/react";
import { cleanup, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ucError } from "../testing/fake-uc.js";
import { type FakeUc, renderWithUc, screen } from "../testing/render.js";
import { PermissionsTab } from "./PermissionsTab.js";

afterEach(cleanup);

const GRANTS_PATH = "/permissions/table/main.sales.orders";

/** Serve `assignments` for the grants GET and the PATCH answer. */
function serveGrants(
  uc: FakeUc,
  assignments: { principal: string; privileges: string[] }[],
) {
  uc.respond((r) =>
    r.path === GRANTS_PATH
      ? Response.json({ privilege_assignments: assignments })
      : undefined,
  );
}

function renderTab(
  assignments: { principal: string; privileges: string[] }[],
  wrap: (ui: React.ReactNode) => React.ReactNode = (ui) => ui,
) {
  return renderWithUc(
    wrap(<PermissionsTab securableType="table" fullName="main.sales.orders" />),
    { setup: (uc) => serveGrants(uc, assignments) },
  );
}

describe("PermissionsTab", () => {
  test("lists assignments in OSS spelling, owner first", async () => {
    const { uc } = renderTab([
      { principal: "data-eng", privileges: ["SELECT", "MODIFY"] },
      { principal: "alice@example.com", privileges: ["SELECT", "OWNER"] },
    ]);
    const alice = (await screen.findByText("alice@example.com")).closest("tr");
    if (!alice) throw new Error("no row");
    const badges = within(alice).getAllByRole("listitem");
    expect(badges.map((b) => b.textContent)).toEqual(["OWNER", "SELECT"]);
    expect(uc.requests[0]).toMatchObject({ method: "GET", path: GRANTS_PATH });
  });

  test("OWNER can't be revoked; other privileges can", async () => {
    const { uc } = renderTab([
      { principal: "alice@example.com", privileges: ["OWNER", "SELECT"] },
    ]);
    const user = userEvent.setup();
    await screen.findByText("alice@example.com");
    expect(
      screen.queryByRole("button", {
        name: "Revoke OWNER from alice@example.com",
      }),
    ).toBeNull();
    await user.click(
      screen.getByRole("button", {
        name: "Revoke SELECT from alice@example.com",
      }),
    );
    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "PATCH",
      path: GRANTS_PATH,
      body: {
        changes: [{ principal: "alice@example.com", remove: ["SELECT"] }],
      },
    });
  });

  test("grant sends the chosen privileges and shows the server's answer", async () => {
    const { uc } = renderTab([]);
    const user = userEvent.setup();
    await screen.findByText(/No privileges are granted directly/);

    uc.respond((r) =>
      r.method === "PATCH"
        ? Response.json({
            privilege_assignments: [
              {
                principal: "bob@example.com",
                privileges: ["MODIFY", "SELECT"],
              },
            ],
          })
        : undefined,
    );
    await user.click(screen.getByRole("button", { name: "Grant" }));
    const dialog = await screen.findByRole("dialog");
    await user.type(
      within(dialog).getByLabelText("Principal"),
      "bob@example.com",
    );
    // Ticked out of menu order; sent in menu order.
    await user.click(within(dialog).getByLabelText("MODIFY"));
    await user.click(within(dialog).getByLabelText("SELECT"));
    await user.click(within(dialog).getByRole("button", { name: "Grant" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0].body).toEqual({
      changes: [{ principal: "bob@example.com", add: ["SELECT", "MODIFY"] }],
    });
    // The PATCH response replaces the cache: no refetch needed.
    expect(await screen.findByText("bob@example.com")).toBeDefined();
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  test("only privileges that apply to the securable type are offered", async () => {
    renderTab([]);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Grant" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByLabelText("SELECT")).toBeDefined();
    expect(within(dialog).queryByLabelText("USE CATALOG")).toBeNull();
    expect(within(dialog).queryByLabelText("OWNER")).toBeNull();
  });

  test("a rejected grant keeps the dialog open with the server's message", async () => {
    const { uc } = renderTab([]);
    const user = userEvent.setup();
    uc.respond((r) =>
      r.method === "PATCH"
        ? ucError(400, "INVALID_PARAMETER_VALUE", "unknown principal 'nobody'")
        : undefined,
    );
    await user.click(await screen.findByRole("button", { name: "Grant" }));
    const dialog = await screen.findByRole("dialog");
    await user.type(within(dialog).getByLabelText("Principal"), "nobody");
    await user.click(within(dialog).getByLabelText("SELECT"));
    await user.click(within(dialog).getByRole("button", { name: "Grant" }));

    expect((await within(dialog).findByRole("alert")).textContent).toContain(
      "unknown principal 'nobody'",
    );
    expect(screen.getByRole("dialog")).toBeDefined();
  });

  test("grant needs a principal and a privilege", async () => {
    renderTab([]);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Grant" }));
    const dialog = await screen.findByRole("dialog");
    const submit = within(dialog).getByRole("button", { name: "Grant" });
    expect((submit as HTMLButtonElement).disabled).toBe(true);
    await user.type(within(dialog).getByLabelText("Principal"), "bob");
    expect((submit as HTMLButtonElement).disabled).toBe(true);
    await user.click(within(dialog).getByLabelText("SELECT"));
    expect((submit as HTMLButtonElement).disabled).toBe(false);
  });

  test("a caller who may not see grants gets a notice, not an error", async () => {
    renderWithUc(
      <PermissionsTab securableType="table" fullName="main.sales.orders" />,
      {
        setup: (uc) =>
          uc.respond((r) =>
            r.path === GRANTS_PATH
              ? ucError(403, "PERMISSION_DENIED", "denied")
              : undefined,
          ),
      },
    );
    expect(
      await screen.findByText(
        /don't have permission to view the grants on this table/,
      ),
    ).toBeDefined();
    expect(screen.queryByRole("button", { name: "Grant" })).toBeNull();
  });

  test("the principal field suggests from the identity directory", async () => {
    const queries: string[] = [];
    const identity: IdentityHost = {
      search: async (query) => {
        queries.push(query);
        return [
          { kind: "group", name: "data-eng", displayName: "Data engineering" },
        ];
      },
    };
    const { uc } = renderTab([], (ui) => (
      <IdentityProvider identity={identity}>{ui}</IdentityProvider>
    ));
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Grant" }));
    const dialog = await screen.findByRole("dialog");
    await user.type(within(dialog).getByLabelText("Principal"), "data");
    await user.click(
      await within(dialog).findByRole("button", { name: /Data engineering/ }),
    );
    expect(
      (within(dialog).getByLabelText("Principal") as HTMLInputElement).value,
    ).toBe("data-eng");
    // Debounced: one search for the settled text, not one per keystroke.
    expect(queries).toEqual(["data"]);

    await user.click(within(dialog).getByLabelText("SELECT"));
    await user.click(within(dialog).getByRole("button", { name: "Grant" }));
    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0].body).toEqual({
      changes: [{ principal: "data-eng", add: ["SELECT"] }],
    });
  });
});
