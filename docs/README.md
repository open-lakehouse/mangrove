# unitycatalog-rs docs

An [Astro](https://astro.build) + [Starlight](https://starlight.astro.build)
site documenting the **architecture and internals** of `unitycatalog-rs`: how
the server is composed, the graph data model, the code-generation pipeline,
authorization, the core trait contracts, and the contributor workflow.

User-facing docs (tutorials, client how-to guides, and REST/configuration
reference) are maintained in the consolidated open-lakehouse documentation, not
here.

## Structure

Content lives in `src/content/docs/`, organized by Diátaxis bucket:

- `explanation/` — architecture and design (service composition, graph data model, codegen, authorization)
- `guides/` — contributor how-tos (`add-resource-type`, `integration-testing`)
- `reference/` — trait references (`trait-policy`, `trait-resource-store`)

## Commands

Run from this `docs/` directory:

| Command           | Action                                        |
| :---------------- | :-------------------------------------------- |
| `bun install`     | Install dependencies                          |
| `bun run dev`     | Start the local dev server at `localhost:4321` |
| `bun run build`   | Build the production site to `./dist/`        |
| `bun run preview` | Preview the build locally                     |
