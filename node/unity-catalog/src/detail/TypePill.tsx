import { Badge } from "@open-lakehouse/ui-kit";

// A securable's storage-management type as a pill (MANAGED / EXTERNAL, and the
// view-like table types). MANAGED gets the green "success" accent — it's the
// default, fully-governed state — while everything else uses the neutral-blue
// "primary" accent. Shared by table and volume detail grids.
export function TypePill({ value }: { value?: string }) {
  if (!value) return <span className="text-muted-foreground">—</span>;
  return (
    <Badge variant={value === "MANAGED" ? "success" : "primary"}>{value}</Badge>
  );
}
