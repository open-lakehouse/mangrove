import * as React from "react";

import { cn } from "./utils";

const Input = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, type, ...props }, ref) => (
  <input
    type={type}
    ref={ref}
    // Identifier/data fields: suppress the browser's text-correction so it
    // doesn't capitalize or "fix" what the user typed. Listed before {...props}
    // so a specific field can opt back in (e.g. a free-text comment).
    autoCapitalize="off"
    autoCorrect="off"
    spellCheck={false}
    className={cn(
      "flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50",
      className,
    )}
    {...props}
  />
));
Input.displayName = "Input";

export { Input };
