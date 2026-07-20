import type { Meta, StoryObj } from "@storybook/react-vite";

import { ActionsLog } from "./actions-log";
import * as arrow from "./story-fixtures";

// ActionsLog reads zero-copy from an ArrowResultStore built from real Arrow IPC:
// six nullable action slots, exactly one non-null per row (the reconciled action
// stream). Each row renders a color+glyph+label badge and inline key fields;
// clicking a row expands the full leaf detail.
const meta: Meta<typeof ActionsLog> = {
  title: "Data/ActionsLog",
  component: ActionsLog,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <div className="h-[480px] w-full overflow-hidden rounded-md border">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof ActionsLog>;

export const ActionStream: Story = {
  args: {
    store: arrow.storeFromIpc(arrow.actionsIpc),
    version: 1,
    running: false,
  },
};

export const Empty: Story = {
  args: {
    store: arrow.storeFromIpc(),
    version: 1,
    running: false,
  },
};
