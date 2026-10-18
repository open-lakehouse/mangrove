// Public surface of the shared UI kit: the shadcn/Radix primitives, the `cn`
// class-merge helper, and the theme context. Both feature packages
// (@open-lakehouse/data-grid, @open-lakehouse/unity-catalog-ui) and the app
// (@open-lakehouse/ui) depend on this package rather than each carrying its own
// copy of the primitives. Distribute visuals from one place; see ./README.md.

export { Badge, type BadgeProps, badgeVariants } from "./badge.js";
export { Button, type ButtonProps, buttonVariants } from "./button.js";
export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
} from "./dialog.js";
export {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu.js";
export { Input } from "./input.js";
export { Label } from "./label.js";
export {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./select.js";
export { Separator } from "./separator.js";
export { Toaster } from "./sonner.js";
export { Switch, type SwitchProps } from "./switch.js";
export { ThemeProvider, useTheme } from "./ThemeProvider.js";
export { Tabs, TabsContent, TabsList, TabsTrigger } from "./tabs.js";
export { Textarea } from "./textarea.js";
export {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip.js";
export { cn } from "./utils.js";
