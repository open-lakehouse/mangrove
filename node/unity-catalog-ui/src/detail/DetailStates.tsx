import { parseUcError } from "@open-lakehouse/unity-catalog-client";

export function DetailStates({
  isLoading,
  error,
}: {
  isLoading: boolean;
  error: unknown;
}) {
  if (error) {
    return <p className="text-sm text-destructive">{parseUcError(error)}</p>;
  }
  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <span className="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-muted border-t-primary" />
        Loading…
      </div>
    );
  }
  return <p className="text-sm text-muted-foreground">Not found.</p>;
}
