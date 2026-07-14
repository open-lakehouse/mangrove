// Public surface of the shared UI kit: the shadcn/Radix primitives, the `cn`
// class-merge helper, and the theme context. Both feature packages
// (@open-lakehouse/data-grid, @open-lakehouse/unity-catalog) and the app
// (@open-lakehouse/ui) depend on this package rather than each carrying its own
// copy of the primitives. Distribute visuals from one place; see ./README.md.

export { Badge, type BadgeProps, badgeVariants } from "./badge";
export { Button, type ButtonProps, buttonVariants } from "./button";
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
} from "./dialog";
export {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./dropdown-menu";
export { Input } from "./input";
export { Label } from "./label";
export {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./select";
export { Separator } from "./separator";
export { Toaster } from "./sonner";
export { Switch, type SwitchProps } from "./switch";
export { ThemeProvider, useTheme } from "./ThemeProvider";
export { Textarea } from "./textarea";
export {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./tooltip";
export { cn } from "./utils";
