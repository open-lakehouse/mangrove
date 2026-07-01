import type { Meta, StoryObj } from "@storybook/react-vite";

import { CatalogRouterHarness } from "../../ui/.storybook/story-router";
import { CatalogExplorer } from "./CatalogExplorer";

// The explorer is hook-driven: it reads catalogs/schemas/tables through the UC
// query layer, which the Storybook host serves from fixtures via the fixture
// fetch. The selection lives in the `/catalog` route, so stories wrap the
// component in the catalog router harness; `initialSel` deep-links a detail view.
const meta: Meta<typeof CatalogExplorer> = {
  title: "Catalog/CatalogExplorer",
  component: CatalogExplorer,
  parameters: { layout: "fullscreen" },
};

export default meta;
type Story = StoryObj<typeof CatalogExplorer>;

export const Default: Story = {
  render: () => (
    <CatalogRouterHarness>
      <CatalogExplorer />
    </CatalogRouterHarness>
  ),
};

export const TableSelected: Story = {
  render: () => (
    <CatalogRouterHarness initialSel="table:main.sales.orders">
      <CatalogExplorer />
    </CatalogRouterHarness>
  ),
};

export const SchemaSelected: Story = {
  render: () => (
    <CatalogRouterHarness initialSel="schema:main.sales">
      <CatalogExplorer />
    </CatalogRouterHarness>
  ),
};

export const VolumeSelected: Story = {
  render: () => (
    <CatalogRouterHarness initialSel="volume:main.sales.raw_files">
      <CatalogExplorer />
    </CatalogRouterHarness>
  ),
};
