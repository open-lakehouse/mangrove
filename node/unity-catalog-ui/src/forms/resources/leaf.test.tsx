import "../../testing/dom.js";

import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreateEntityDialog } from "../../CreateEntityDialog.js";
import { EditEntityDialog } from "../../EditEntityDialog.js";
import { renderWithUc, screen } from "../../testing/render.js";

afterEach(cleanup);

const noop = () => {};

describe("create volume", () => {
  test("a managed volume sends no storage location", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "volume", catalog: "main", schema: "sales" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "exports");
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "POST",
      path: "/volumes",
      body: {
        name: "exports",
        catalog_name: "main",
        schema_name: "sales",
        volume_type: "MANAGED",
      },
    });
    expect(uc.writes()[0]?.body).not.toHaveProperty("storage_location");
  });

  test("an external volume needs a storage location", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "volume", catalog: "main", schema: "sales" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "landing");
    await user.click(screen.getByLabelText(/^Volume type/));
    await user.click(await screen.findByRole("option", { name: /^External/ }));
    await user.click(screen.getByRole("button", { name: "Create" }));

    expect(
      await screen.findByText("Required for an external volume."),
    ).toBeDefined();
    expect(uc.writes()).toHaveLength(0);

    await user.type(
      screen.getByLabelText(/^Storage location/),
      "s3://bucket/landing",
    );
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]?.body).toMatchObject({
      volume_type: "EXTERNAL",
      storage_location: "s3://bucket/landing",
    });
  });

  test("the parent schema is required", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "volume", catalog: "main" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "exports");
    await user.click(screen.getByRole("button", { name: "Create" }));

    expect(await screen.findByText("Required.")).toBeDefined();
    expect(uc.writes()).toHaveLength(0);
  });
});

describe("registered models", () => {
  test("create sends the parent and name", async () => {
    const { uc } = renderWithUc(
      <CreateEntityDialog
        request={{ kind: "model", catalog: "main", schema: "sales" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "churn");
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "POST",
      path: "/models",
      body: { name: "churn", catalog_name: "main", schema_name: "sales" },
    });
  });

  test("edit renames by full name", async () => {
    const { uc } = renderWithUc(
      <EditEntityDialog
        request={{ kind: "volume", name: "main.sales.raw_files" }}
        onClose={noop}
      />,
    );
    const user = userEvent.setup();

    const name = await screen.findByLabelText(/^Name/);
    await waitFor(() =>
      expect((name as HTMLInputElement).value).toBe("raw_files"),
    );
    await user.clear(name);
    await user.type(name, "landing_files");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "PATCH",
      path: "/volumes/main.sales.raw_files",
      body: { new_name: "landing_files" },
    });
  });
});
