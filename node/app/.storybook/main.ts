import type { StorybookConfig } from "@storybook/react-vite";

// One Storybook for the node/ workspace's UI packages, themed by the app's
// token values (src/globals.css). The builder reuses this app's vite.config.ts,
// so the `source` condition, Tailwind and the no-op wasm stubs apply here too:
// stories run on fixtures alone, with no wasm build and no services.
const config: StorybookConfig = {
  framework: "@storybook/react-vite",
  stories: [
    "../../unity-catalog-ui/src/**/*.stories.tsx",
    "../../data-grid/src/**/*.stories.tsx",
    "../../editor/src/**/*.stories.tsx",
    "../../log-query/src/**/*.stories.tsx",
    "../../query/src/**/*.stories.tsx",
  ],
  core: { disableTelemetry: true },
};

export default config;
