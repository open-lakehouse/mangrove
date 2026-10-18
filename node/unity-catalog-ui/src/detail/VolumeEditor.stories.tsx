import { FilesServiceProvider } from "@open-lakehouse/files";
import { registerStubFiles } from "@open-lakehouse/files/testing";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { VolumeEditor } from "./VolumeEditor";

// The Files tab is hook-driven off the @open-lakehouse/files seam. Register the
// dev stub runner (the rich nested fixture tree under /Volumes/demo/raw/events)
// once at module load — the same wiring the app does in non-wasm builds — and
// register NO query runner, so the SQL results pane stays hidden and the story
// exercises pure file browsing: drill-down, breadcrumb, up-button, rows, download.
registerStubFiles();

const meta: Meta<typeof VolumeEditor> = {
  title: "Catalog/VolumeEditor",
  component: VolumeEditor,
  parameters: { layout: "padded" },
  decorators: [
    (Story) => (
      <FilesServiceProvider>
        <Story />
      </FilesServiceProvider>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof VolumeEditor>;

// fullName maps to the fixture root /Volumes/demo/raw/events. Open the Files
// list, drill into `date=2026-05-01` → `_metadata`, click a crumb to jump back,
// use the up button, and download a file to save the fixture bytes.
export const Default: Story = {
  render: () => <VolumeEditor fullName="demo.raw.events" />,
};
