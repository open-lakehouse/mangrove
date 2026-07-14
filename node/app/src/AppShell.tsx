import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  useTheme,
} from "@open-lakehouse/ui-kit";
import { Outlet } from "@tanstack/react-router";
import { Monitor, Moon, Sun } from "lucide-react";

import { UnityCatalogIcon } from "./UnityCatalogIcon";

// Global app chrome: a slim top bar (brand on the left, actions on the right)
// above the routed content. The header is a fixed 3rem tall and the content
// region flexes to fill the rest of the viewport — CatalogExplorer inside it
// uses h-full, so the two stay in lockstep and nothing gets clipped.
export function AppShell() {
  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <header className="flex h-12 shrink-0 items-center justify-between border-b bg-card px-4">
        <div className="flex items-center gap-2 text-foreground">
          {/* Subtle, monochrome brand mark: muted so it reads as chrome, not a
              call-to-action, matching the console theme. */}
          <UnityCatalogIcon className="h-5 w-5 text-muted-foreground" />
          <span className="text-sm font-semibold tracking-tight">
            Unity Catalog
          </span>
        </div>
        <div className="flex items-center gap-1">
          <ThemeToggle />
        </div>
      </header>
      <div className="min-h-0 flex-1">
        <Outlet />
      </div>
    </div>
  );
}

const THEMES = [
  { value: "light", label: "Light", Icon: Sun },
  { value: "dark", label: "Dark", Icon: Moon },
  { value: "system", label: "System", Icon: Monitor },
] as const;

function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const Active = (THEMES.find((t) => t.value === theme) ?? THEMES[2]).Icon;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          aria-label="Toggle theme"
          title="Toggle theme"
        >
          <Active className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {THEMES.map(({ value, label, Icon }) => (
          <DropdownMenuItem
            key={value}
            onSelect={() => setTheme(value)}
            className={theme === value ? "text-foreground" : undefined}
          >
            <Icon />
            {label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
