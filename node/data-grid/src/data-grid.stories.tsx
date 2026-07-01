import type { Meta, StoryObj } from "@storybook/react-vite";

import { DataGrid } from "./data-grid";
import * as arrow from "./story-fixtures";

// The grid reads zero-copy from an ArrowResultStore built from real Arrow IPC
// fixtures — the same bytes the streaming QueryService would deliver.
const meta: Meta<typeof DataGrid> = {
  title: "Data/DataGrid",
  component: DataGrid,
  parameters: { layout: "fullscreen" },
  // Give the virtualized grid a bounded height to scroll within.
  decorators: [
    (Story) => (
      <div className="h-[420px] w-full border rounded-md overflow-hidden">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof DataGrid>;

export const TopCustomers: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.topCustomersIpc),
    version: 1,
    running: false,
  },
};

export const WideResult: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.tripsIpc),
    version: 1,
    running: false,
  },
};

export const Empty: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.emptyIpc),
    version: 1,
    running: false,
  },
};

// While a query streams, sorting is gated until the stream ends.
export const Streaming: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.topCustomersIpc),
    version: 1,
    running: true,
  },
};
