import type { IdentityHost } from "@open-lakehouse/unity-catalog-client";
import { IdentityProvider } from "@open-lakehouse/unity-catalog-client/react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { PermissionsTab } from "./PermissionsTab.js";

// Grants are read from the Storybook host's fixture catalog (the owner's OWNER
// plus a few fixture grants). The fixture is read-only, so granting or revoking
// shows the server's error toast.
const meta: Meta<typeof PermissionsTab> = {
  title: "Catalog/PermissionsTab",
  component: PermissionsTab,
  parameters: { layout: "padded" },
};

export default meta;
type Story = StoryObj<typeof PermissionsTab>;

const directory: IdentityHost = {
  async search(query) {
    const q = query.toLowerCase();
    return [
      {
        kind: "user" as const,
        name: "alice@example.com",
        displayName: "Alice",
      },
      { kind: "user" as const, name: "bob@example.com", displayName: "Bob" },
      { kind: "group" as const, name: "analysts" },
      { kind: "service_account" as const, name: "etl-nightly" },
    ].filter(
      (p) =>
        p.name.includes(q) || (p.displayName ?? "").toLowerCase().includes(q),
    );
  },
};

/** Without an identity directory: the principal is typed by name. */
export const Catalog: Story = {
  args: { securableType: "catalog", fullName: "main" },
};

/** With a host directory: the principal field suggests as you type. */
export const WithDirectory: Story = {
  args: { securableType: "schema", fullName: "main.sales" },
  decorators: [
    (Story) => (
      <IdentityProvider identity={directory}>
        <Story />
      </IdentityProvider>
    ),
  ],
};
