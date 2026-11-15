import "../testing/dom.js";

import { afterEach, describe, expect, mock, test } from "bun:test";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import {
  canSubmitGrant,
  GrantForm,
  type GrantFormSubmission,
  type GrantFormValue,
  grantSubmission,
} from "./GrantForm.js";

afterEach(cleanup);

function Harness({
  initial = { principal: "", privileges: [] },
  onSubmit,
  pending,
  error,
}: {
  initial?: GrantFormValue;
  onSubmit: (grant: GrantFormSubmission) => void;
  pending?: boolean;
  error?: string;
}) {
  const [value, setValue] = useState(initial);
  return (
    <GrantForm
      securableType="schema"
      value={value}
      onChange={setValue}
      onSubmit={onSubmit}
      pending={pending}
      error={error}
    />
  );
}

describe("grantSubmission", () => {
  test("trims the principal and keeps only applicable privileges, in menu order", () => {
    expect(
      grantSubmission("schema", {
        principal: "  bob@example.com ",
        // CREATE_CATALOG doesn't apply to a schema.
        privileges: ["SELECT", "CREATE_CATALOG", "USE_SCHEMA"],
      }),
    ).toEqual({ principal: "bob@example.com", add: ["USE_SCHEMA", "SELECT"] });
  });

  test("needs a principal and an applicable privilege", () => {
    expect(
      canSubmitGrant("schema", { principal: " ", privileges: ["SELECT"] }),
    ).toBe(false);
    expect(
      canSubmitGrant("schema", {
        principal: "bob",
        privileges: ["CREATE_CATALOG"],
      }),
    ).toBe(false);
    expect(
      canSubmitGrant("schema", { principal: "bob", privileges: ["SELECT"] }),
    ).toBe(true);
  });
});

describe("GrantForm", () => {
  test("offers only the privileges that apply to the securable type", () => {
    render(<Harness onSubmit={() => {}} />);
    expect(screen.getByRole("checkbox", { name: "USE SCHEMA" })).toBeTruthy();
    expect(
      screen.queryByRole("checkbox", { name: "CREATE CATALOG" }),
    ).toBeNull();
  });

  test("submits the prefilled value in menu order", async () => {
    const onSubmit = mock((_: GrantFormSubmission) => {});
    render(
      <Harness
        initial={{ principal: "data-eng", privileges: ["SELECT"] }}
        onSubmit={onSubmit}
      />,
    );
    const user = userEvent.setup();
    await user.click(screen.getByRole("checkbox", { name: "USE SCHEMA" }));
    await user.click(screen.getByRole("button", { name: "Grant" }));
    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit.mock.calls[0]?.[0]).toEqual({
      principal: "data-eng",
      add: ["USE_SCHEMA", "SELECT"],
    });
  });

  test("can't submit without a privilege, or while pending", async () => {
    const { rerender } = render(
      <Harness
        initial={{ principal: "bob", privileges: [] }}
        onSubmit={() => {}}
      />,
    );
    const button = () =>
      screen.getByRole("button", { name: /Grant/ }) as HTMLButtonElement;
    expect(button().disabled).toBe(true);
    rerender(
      <Harness
        initial={{ principal: "bob", privileges: ["SELECT"] }}
        onSubmit={() => {}}
        pending
      />,
    );
    expect(button().disabled).toBe(true);
    expect(button().textContent).toBe("Granting…");
  });

  test("shows the error", () => {
    render(<Harness onSubmit={() => {}} error="unknown principal" />);
    expect(screen.getByRole("alert").textContent).toBe("unknown principal");
  });
});
