# `@open-lakehouse/ui-kit`

The shared presentational kit for the Open Lakehouse web UIs: the shadcn/Radix
primitives (`Button`, `Dialog`, `Select`, `Tooltip`, …), the `cn` class-merge
helper, and the `ThemeProvider` / `useTheme` theme context.

The feature packages (`@open-lakehouse/data-grid`,
`@open-lakehouse/unity-catalog`), the mangrove scaffold app, and downstream
consumers (hydrofoil's unified UI, via the sibling-repo `file:` link) all depend
on this package, so the primitives live in exactly one place instead of being
copied per consumer.

## Public surface

Import from the barrel — `@open-lakehouse/ui-kit`:

- Primitives: `Badge`, `Button`, `Dialog*`, `DropdownMenu*`, `Input`, `Label`,
  `Select*`, `Separator`, `Textarea`, `Toaster`, `Tooltip*` (+ the `*Variants`
  and `*Props` for `Badge`/`Button`).
- Helpers: `cn`.
- Theme: `ThemeProvider`, `useTheme`.

## Tailwind

This package ships Tailwind utility classes but owns no Tailwind config and no
theme values. The consuming app's Tailwind build must scan the package source —
the app adds a `@source` glob pointing at this package's `src` in its global
stylesheet — and the app supplies the design tokens. The token contract every
consumer must satisfy is documented in [`DESIGN.md`](./DESIGN.md).

## Distribution

Source-only workspace package (`exports` points straight at `src/index.ts`), so
the consuming app's Vite/tsc compile it directly — no build step. During the
current evaluation phase nothing is published; sibling repos consume it via a
local `file:` link across checkouts.
