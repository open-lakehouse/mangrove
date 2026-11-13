import "../../testing/dom.js";

import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreateEntityDialog } from "../../CreateEntityDialog.js";
import { EditEntityDialog } from "../../EditEntityDialog.js";
import { ucError } from "../../testing/fake-uc.js";
import { renderWithUc, screen } from "../../testing/render.js";

afterEach(cleanup);

const noop = () => {};

describe("create schema", () => {
  test("sends the picked catalog with the name", async () => {
    let closed = false;
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "schema", catalog: "main" }}
        onClose={() => {
          closed = true;
        }}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "bronze");
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "POST",
      path: "/schemas",
      body: { name: "bronze", catalog_name: "main" },
    });
    await waitFor(() => expect(closed).toBe(true));
  });

  test("an invalid name is shown on the field and nothing is sent", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "schema", catalog: "main" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "ab");
    await user.click(screen.getByRole("button", { name: "Create" }));

    expect(await screen.findByText(/at least 3 characters/)).toBeDefined();
    expect(uc.writes()).toHaveLength(0);
  });

  test("a server error stays in the dialog", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "schema", catalog: "main" }}
        onClose={noop}
      />,
    );
    uc.respond((r) =>
      r.method === "POST"
        ? ucError(409, "ALREADY_EXISTS", "Schema main.bronze already exists")
        : undefined,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "bronze");
    await user.click(screen.getByRole("button", { name: "Create" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("already exists");
    expect(screen.getByRole("button", { name: "Create" })).toBeDefined();
  });

  test("properties are sent as a map", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "schema", catalog: "main" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "bronze");
    await user.click(screen.getByRole("button", { name: /Add property/ }));
    await user.type(screen.getByLabelText("Key"), "owner_team");
    await user.type(screen.getByLabelText("Value"), "data");
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]?.body).toEqual({
      name: "bronze",
      catalog_name: "main",
      properties: { owner_team: "data" },
    });
  });
});

describe("create catalog", () => {
  test("sends only what the user filled in", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog request={{ kind: "catalog" }} onClose={noop} />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "Sales Data");
    await user.type(screen.getByLabelText("Comment"), "Revenue");
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    // Typed capitals and spaces become a valid identifier.
    expect(uc.writes()[0]?.body).toEqual({
      name: "sales_data",
      comment: "Revenue",
    });
  });
});

describe("edit schema", () => {
  test("editing the comment sends only the comment", async () => {
    const { uc } = renderWithUc(
      <EditEntityDialog
        request={{ kind: "schema", name: "main.sales" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    const comment = await screen.findByLabelText("Comment");
    await waitFor(() =>
      expect((comment as HTMLTextAreaElement).value).toBe(
        "Orders and invoices",
      ),
    );
    await user.type(comment, ", archived");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "PATCH",
      path: "/schemas/main.sales",
      body: { comment: "Orders and invoices, archived" },
    });
  });

  test("a rename sends new_name only", async () => {
    const { uc } = renderWithUc(
      <EditEntityDialog
        request={{ kind: "schema", name: "main.sales" }}
        onClose={noop}
      />,
      { initialSel: "schema:main.sales" },
    );
    const user = userEvent.setup();

    const name = await screen.findByLabelText(/^Name/);
    await waitFor(() => expect((name as HTMLInputElement).value).toBe("sales"));
    await user.clear(name);
    await user.type(name, "revenue");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]?.body).toEqual({ new_name: "revenue" });
  });

  test("saving without changes sends nothing", async () => {
    let closed = false;
    const { uc } = renderWithUc(
      <EditEntityDialog
        request={{ kind: "schema", name: "main.sales" }}
        onClose={() => {
          closed = true;
        }}
      />,
    );
    const user = userEvent.setup();

    const name = await screen.findByLabelText(/^Name/);
    await waitFor(() => expect((name as HTMLInputElement).value).toBe("sales"));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(closed).toBe(true));
    expect(uc.writes()).toHaveLength(0);
  });

  test("a failed load is shown instead of the form", async () => {
    renderWithUc(
      <EditEntityDialog
        request={{ kind: "schema", name: "main.missing" }}
        onClose={noop}
      />,
    );
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("does not exist");
    expect(
      (screen.getByRole("button", { name: "Save" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });
});
