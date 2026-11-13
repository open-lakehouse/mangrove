import "../testing/dom.js";

import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithUc, screen } from "../testing/render.js";
import { CredentialDialog } from "./CredentialDialog.js";
import { ExternalLocationDialog } from "./ExternalLocationDialog.js";

afterEach(cleanup);

const noop = () => {};

const azureCredential = {
  name: "azure_cred",
  purpose: "STORAGE",
  comment: "Lake access",
  azure_managed_identity: {
    access_connector_id: "/subscriptions/x/accessConnectors/c",
    credential_id: "server-assigned",
  },
};

describe("credentials", () => {
  test("create sends only the chosen cloud configuration", async () => {
    const { uc } = renderWithUc(
      <CredentialDialog mode="create" onClose={noop} />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "lake_role");
    await user.type(
      screen.getByLabelText(/^IAM role ARN/),
      "arn:aws:iam::123456789012:role/uc",
    );
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "POST",
      path: "/credentials",
      body: {
        name: "lake_role",
        purpose: "STORAGE",
        aws_iam_role: { role_arn: "arn:aws:iam::123456789012:role/uc" },
      },
    });
  });

  test("the chosen type's required fields are enforced", async () => {
    const { uc } = renderWithUc(
      <CredentialDialog mode="create" onClose={noop} />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "lake_role");
    await user.click(screen.getByRole("button", { name: "Create" }));

    expect(await screen.findByText("Required.")).toBeDefined();
    expect(uc.writes()).toHaveLength(0);
  });

  test("switching type drops the previous configuration", async () => {
    const { uc } = renderWithUc(
      <CredentialDialog mode="create" onClose={noop} />,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "azure_cred");
    await user.type(screen.getByLabelText(/^IAM role ARN/), "arn:x");
    await user.click(screen.getByLabelText("Credential type"));
    await user.click(
      await screen.findByRole("option", { name: "Azure managed identity" }),
    );
    await user.type(
      screen.getByLabelText(/^Access connector id/),
      "/subscriptions/x/accessConnectors/c",
    );
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    const body = uc.writes()[0]?.body as Record<string, unknown>;
    expect(body.azure_managed_identity).toEqual({
      access_connector_id: "/subscriptions/x/accessConnectors/c",
    });
    expect(body).not.toHaveProperty("aws_iam_role");
  });

  test("editing a non-AWS credential never sends an AWS role", async () => {
    const { uc } = renderWithUc(
      <CredentialDialog mode="edit" name="azure_cred" onClose={noop} />,
    );
    uc.respond((r) =>
      r.method === "GET" && r.path === "/credentials/azure_cred"
        ? Response.json(azureCredential)
        : undefined,
    );
    const user = userEvent.setup();

    const name = await screen.findByLabelText(/^Name/);
    await waitFor(() =>
      expect((name as HTMLInputElement).value).toBe("azure_cred"),
    );
    // The type is shown but fixed, and output-only fields aren't editable.
    expect(screen.getByText("Azure managed identity")).toBeDefined();
    expect(screen.queryByDisplayValue("server-assigned")).toBeNull();

    await user.clear(name);
    await user.type(name, "azure_lake");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "PATCH",
      path: "/credentials/azure_cred",
      body: { new_name: "azure_lake" },
    });
    expect(uc.writes()[0]?.body).not.toHaveProperty("aws_iam_role");
    expect(uc.writes()[0]?.body).not.toHaveProperty("purpose");
  });
});

describe("external locations", () => {
  test("create sends the picked credential", async () => {
    const { uc } = renderWithUc(
      <ExternalLocationDialog mode="create" onClose={noop} />,
    );
    uc.respond((r) =>
      r.method === "GET" && r.path === "/credentials"
        ? Response.json({ credentials: [{ name: "lake_role" }] })
        : undefined,
    );
    const user = userEvent.setup();

    await user.type(await screen.findByLabelText(/^Name/), "landing");
    await user.type(screen.getByLabelText(/^URL/), "s3://bucket/landing");
    await user.click(screen.getByLabelText(/^Credential/));
    await user.click(await screen.findByRole("option", { name: "lake_role" }));
    await user.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "POST",
      path: "/external-locations",
      body: {
        name: "landing",
        url: "s3://bucket/landing",
        credential_name: "lake_role",
      },
    });
  });

  test("edit sends only the changed URL", async () => {
    const { uc } = renderWithUc(
      <ExternalLocationDialog mode="edit" name="landing" onClose={noop} />,
    );
    uc.respond((r) =>
      r.method === "GET" && r.path === "/external-locations/landing"
        ? Response.json({
            name: "landing",
            url: "s3://bucket/landing",
            credential_name: "lake_role",
          })
        : undefined,
    );
    const user = userEvent.setup();

    const url = await screen.findByLabelText(/^URL/);
    await waitFor(() =>
      expect((url as HTMLInputElement).value).toBe("s3://bucket/landing"),
    );
    await user.clear(url);
    await user.type(url, "s3://bucket/landing_v2");
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(uc.writes()).toHaveLength(1));
    expect(uc.writes()[0]).toMatchObject({
      method: "PATCH",
      path: "/external-locations/landing",
      body: { url: "s3://bucket/landing_v2" },
    });
  });
});
