# Design token contract — `@open-lakehouse/ui-kit`

This is the shared design contract for every Open Lakehouse web UI (the Unity
Catalog app here, hydrofoil's unified desktop/web shell, and — by the same
convention — headwaters' lineage UI). It follows the
[`design.md`](https://github.com/google-labs-code/design.md) convention: a single
place that fixes the *contract*, not the concrete look.

The primitives in this package are **headless with respect to theme**. They emit
semantic Tailwind utilities (`bg-background`, `text-foreground`, `border-border`,
…) and own **no color values, no light/dark palette, and no Tailwind config**.
Each host application supplies the actual values as CSS variables. Aligning on the
variable *names and meanings* below is what keeps three separate apps visually
coherent while staying independently themeable.

## How theming is injected

1. The host defines the variables below on `:root` (light) and `.dark` (dark) in
   its own global stylesheet.
2. The host maps them to Tailwind color tokens via `@theme inline`
   (`--color-background: hsl(var(--background));` …).
3. The host adds a `@source` glob so Tailwind scans this package's source and
   generates the utilities the primitives reference.

The canonical reference implementation of steps 1–3 is hydrofoil's
`node/ui/src/app/globals.css`; the mangrove scaffold app carries an equivalent.
A host may pick entirely different values — only the variable names are the
contract.

## The variables (HSL components: `H S% L%`)

Every consumer MUST define all of these on both `:root` and `.dark`.

### Surfaces & text
| Variable | Meaning |
| --- | --- |
| `--background` / `--foreground` | App canvas + primary text on it |
| `--card` / `--card-foreground` | Raised surface (panels, cards) + its text |
| `--popover` / `--popover-foreground` | Floating surface (menus, tooltips) + its text |
| `--muted` / `--muted-foreground` | Subdued fill + secondary/label text |

### Interaction
| Variable | Meaning |
| --- | --- |
| `--primary` / `--primary-foreground` | Primary action fill + text on it |
| `--secondary` / `--secondary-foreground` | Secondary action fill + text on it |
| `--accent` / `--accent-foreground` | Hover/active accent fill + text on it |
| `--destructive` / `--destructive-foreground` | Danger action fill + text on it |
| `--border` | Hairline borders (also the default `border-color`) |
| `--input` | Form control border |
| `--ring` | Focus ring |
| `--radius` | Base corner radius (length, e.g. `0.5rem` — not an HSL triple) |

### IDE chrome (sidebar)
| Variable | Meaning |
| --- | --- |
| `--sidebar` / `--sidebar-foreground` | Sidebar surface + text |
| `--sidebar-accent` | Sidebar selected/hover fill |

### Status accents
| Variable | Meaning |
| --- | --- |
| `--status-planned` | Neutral / not started |
| `--status-ready` | Ready / warning-amber |
| `--status-done` | Success |
| `--status-in-progress` | Active (usually tracks `--primary`) |
| `--status-blocked` | Error / blocked |

### Typography (font stacks, not HSL)
| Variable | Meaning |
| --- | --- |
| `--font-sans` | UI sans-serif stack |
| `--font-mono` | Monospace stack (code, identifiers, data cells) |

## Rules

- **Add a token here before using it in a primitive.** A primitive must never
  reference a `--var` that isn't in this contract, or hosts will render it unset.
- **Values live in hosts, not here.** Do not add a `:root {}` block or a
  `tailwind.config` to this package.
- **Changing a token's meaning is a contract change** — coordinate across the
  consuming apps (mangrove app, hydrofoil, headwaters) so palettes stay aligned.
