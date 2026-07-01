import type { Meta, StoryObj } from "@storybook/react-vite";

import { Badge } from "./badge";

const meta: Meta<typeof Badge> = {
  title: "UI/Badge",
  component: Badge,
  args: { children: "Badge" },
};

export default meta;
type Story = StoryObj<typeof Badge>;

export const Default: Story = {};

export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Badge>default</Badge>
      <Badge variant="primary">primary</Badge>
      <Badge variant="outline">outline</Badge>
      <Badge variant="destructive">destructive</Badge>
    </div>
  ),
};

// How badges read in context — e.g. a column's data type or a tag value.
export const AsTypeLabels: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-2">
      <Badge variant="primary">bigint</Badge>
      <Badge variant="primary">string</Badge>
      <Badge variant="primary">timestamp</Badge>
      <Badge variant="outline">nullable</Badge>
    </div>
  ),
};
