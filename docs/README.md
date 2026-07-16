# unitycatalog-rs docs

An [Astro](https://astro.build) + [Starlight](https://starlight.astro.build)
site documenting the **architecture and internals** of `unitycatalog-rs`: its
authorization model and the contributor workflow. Per-item trait/type reference
lives in the crate rustdoc on docs.rs.

User-facing docs (tutorials, client how-to guides, and REST/configuration
reference) are maintained in the consolidated open-lakehouse documentation, not
here.

## Structure

Content lives in `src/content/docs/`, organized by Diátaxis bucket:

- `explanation/` — architecture and design (authorization)
- `guides/` — contributor how-tos (`add-resource-type`, `integration-testing`)

Per-item API reference (the trait/type surface) lives in the crate rustdoc on
[docs.rs](https://docs.rs/olai-uc-server), not on this site — the sidebar links
out to it. Write it as `///` doc comments in the crate, not as Markdown here.

## Commands

Run from this `docs/` directory:

| Command           | Action                                        |
| :---------------- | :-------------------------------------------- |
| `bun install`     | Install dependencies                          |
| `bun run dev`     | Start the local dev server at `localhost:4321` |
| `bun run build`   | Build the production site to `./dist/`        |
| `bun run preview` | Preview the build locally                     |
