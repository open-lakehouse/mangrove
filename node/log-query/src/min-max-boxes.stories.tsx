import type { Meta, StoryObj } from "@storybook/react-vite";
import { MinMaxView } from "./min-max-view";
import * as arrow from "./story-fixtures";

// MinMaxView enumerates orderable min/max axes from the reconciled-log stats and
// plots per-file intervals (1D) / bounding boxes (2D), reading the nested
// stats.minValues/maxValues struct vectors zero-copy.
const meta: Meta<typeof MinMaxView> = {
  title: "Data/MinMaxBoxes",
  component: MinMaxView,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <div className="w-full p-4">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof MinMaxView>;

export const OverlappingFiles: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.reconciledStatsIpc),
    version: 1,
  },
};
