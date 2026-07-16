// @ts-check
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightLlmsTxt from "starlight-llms-txt";

// https://astro.build/config
export default defineConfig({
  site: "https://unitycatalog-incubator.github.io/unitycatalog-rs",
  integrations: [
    starlight({
      title: "Unity Catalog Rust",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/unitycatalog-incubator/unitycatalog-rs",
        },
      ],
      sidebar: [
        {
          label: "Explanation",
          autogenerate: { directory: "explanation" },
        },
        {
          label: "Contributor guides",
          autogenerate: { directory: "guides" },
        },
        {
          label: "Reference",
          autogenerate: { directory: "reference" },
        },
      ],
      plugins: [
        starlightLlmsTxt({
          projectName: "Unity Catalog Rust",
          description:
            "A Rust implementation of the Unity Catalog REST API, with Rust, Python, and TypeScript client libraries. Unity Catalog is an open-source data catalog supporting catalogs, schemas, tables, volumes, and Delta Sharing.",
        }),
      ],
    }),
  ],
});
