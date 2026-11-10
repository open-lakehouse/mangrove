// Stories for the read-only viewers: a file and a file's change, in the
// ui-kit theme (toggle the app theme to check light and dark). Like the other
// stories, the reference harness; excluded from tsc.

import { ThemeProvider } from "@open-lakehouse/ui-kit";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { DiffHost } from "./DiffHost.js";
import { ReadOnlyFile } from "./ReadOnlyFile.js";

const BEFORE = `services:
  postgres:
    image: postgres:18
    ports: ["5432:5432"]
`;
const AFTER = `services:
  postgres:
    image: postgres:18
    ports: ["5432:5432"]
  jaeger:
    image: jaegertracing/jaeger:2
    ports: ["16686:16686"]
`;

function Viewers() {
  return (
    <div style={{ display: "grid", gap: 12 }}>
      <div style={{ height: 200, border: "1px solid var(--border)" }}>
        <ReadOnlyFile path="compose.yaml" value={AFTER} />
      </div>
      <div style={{ height: 240, border: "1px solid var(--border)" }}>
        <DiffHost path="compose.yaml" original={BEFORE} modified={AFTER} />
      </div>
      <div style={{ height: 120, border: "1px solid var(--border)" }}>
        <DiffHost path="opa/data.json" modified={'{ "allow": true }\n'} />
      </div>
    </div>
  );
}

const meta: Meta<typeof Viewers> = {
  title: "editor/Read-only viewers",
  component: Viewers,
  decorators: [
    (Story) => (
      <ThemeProvider>
        <Story />
      </ThemeProvider>
    ),
  ],
};
export default meta;

type Story = StoryObj<typeof Viewers>;

export const FileAndDiff: Story = {};
