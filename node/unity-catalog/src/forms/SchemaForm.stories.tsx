import type { RJSFSchema } from "@rjsf/utils";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { SchemaForm } from "./SchemaForm";
// SchemaForm renders the checked-in JSON Schemas that back the UI's create/edit
// dialogs (generated from the Unity Catalog protos — see
// uc-client/scripts/gen-form-schemas.mjs). These stories load those exact schemas
// so the form here matches the one in the app's dialogs.
import createCatalog from "./schemas/create-catalog.json";
import createCredential from "./schemas/create-credential.json";
import createSchema from "./schemas/create-schema.json";

const meta: Meta<typeof SchemaForm> = {
  title: "Forms/SchemaForm",
  component: SchemaForm,
  parameters: { layout: "padded" },
  decorators: [
    (Story) => (
      <div className="max-w-lg">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof SchemaForm>;

export const CreateCatalog: Story = {
  args: {
    id: "create-catalog",
    schema: createCatalog as RJSFSchema,
    onSubmit: () => {},
  },
};

export const CreateSchema: Story = {
  args: {
    id: "create-schema",
    schema: createSchema as RJSFSchema,
    formData: { catalog_name: "main" },
    onSubmit: () => {},
  },
};

export const CreateCredential: Story = {
  args: {
    id: "create-credential",
    schema: createCredential as RJSFSchema,
    onSubmit: () => {},
  },
};

export const Disabled: Story = {
  args: {
    id: "create-catalog-disabled",
    schema: createCatalog as RJSFSchema,
    formData: { name: "main", comment: "Primary production catalog." },
    disabled: true,
    onSubmit: () => {},
  },
};
