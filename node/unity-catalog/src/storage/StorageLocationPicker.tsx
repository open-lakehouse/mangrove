// Storage-location picker for catalog/schema creation.
//
// Mirrors the database behavior: leaving it on "Managed" lets the securable
// inherit its parent's managed storage, picking a defined external location
// (optionally with a subpath under it) sets an explicit storage root covered by
// that location, and "Custom location" allows entering a storage URL directly
// (useful before any external locations are defined, or for ad-hoc roots).

import {
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@open-lakehouse/ui-kit";
import { useExternalLocations } from "@open-lakehouse/unity-catalog-client";
import { useId, useMemo, useState } from "react";

// Radix Select can't use an empty-string value, so the modes need sentinels.
const MANAGED = "__managed__";
const CUSTOM = "__custom__";

export function StorageLocationPicker({
  onChange,
}: {
  /** Called with the composed storage location, or undefined for managed storage. */
  onChange: (storageLocation: string | undefined) => void;
}) {
  const locations = useExternalLocations();
  const [mode, setMode] = useState(MANAGED);
  const [subpath, setSubpath] = useState("");
  const [customUrl, setCustomUrl] = useState("");

  const selectId = useId();
  const subpathId = useId();
  const urlId = useId();

  const items = useMemo(
    () => (locations.data ?? []).filter((l) => !!l.name && !!l.url),
    [locations.data],
  );

  function compose(nextMode: string, sub: string, custom: string) {
    if (nextMode === MANAGED) return undefined;
    if (nextMode === CUSTOM) return custom.trim() || undefined;
    const base = items.find((l) => l.name === nextMode)?.url;
    if (!base) return undefined;
    const trimmedBase = base.replace(/\/+$/, "");
    const trimmedSub = sub.trim().replace(/^\/+/, "");
    return trimmedSub ? `${trimmedBase}/${trimmedSub}` : trimmedBase;
  }

  function selectMode(next: string) {
    setMode(next);
    onChange(compose(next, subpath, customUrl));
  }

  function changeSubpath(sub: string) {
    setSubpath(sub);
    onChange(compose(mode, sub, customUrl));
  }

  function changeCustomUrl(url: string) {
    setCustomUrl(url);
    onChange(compose(mode, subpath, url));
  }

  const composed = compose(mode, subpath, customUrl);
  const isLocation = mode !== MANAGED && mode !== CUSTOM;

  return (
    <div className="space-y-2 rounded-md border bg-muted/30 p-3">
      <div className="space-y-1.5">
        <Label htmlFor={selectId}>Storage location</Label>
        <Select value={mode} onValueChange={selectMode}>
          <SelectTrigger id={selectId}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={MANAGED}>
              Managed (inherit from parent)
            </SelectItem>
            {items.map((location) => (
              <SelectItem key={location.name} value={location.name ?? ""}>
                {location.name}
              </SelectItem>
            ))}
            <SelectItem value={CUSTOM}>
              Custom location (enter a URL)
            </SelectItem>
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          Choose a defined external location, enter a storage URL, or keep it
          managed to inherit from the parent.
        </p>
      </div>

      {isLocation && (
        <div className="space-y-1.5">
          <Label htmlFor={subpathId}>Subpath (optional)</Label>
          <Input
            id={subpathId}
            value={subpath}
            onChange={(e) => changeSubpath(e.target.value)}
            placeholder="subfolder/path"
          />
        </div>
      )}

      {mode === CUSTOM && (
        <div className="space-y-1.5">
          <Label htmlFor={urlId}>Storage URL</Label>
          <Input
            id={urlId}
            value={customUrl}
            onChange={(e) => changeCustomUrl(e.target.value)}
            placeholder="s3://bucket/path"
          />
        </div>
      )}

      {composed && (
        <p className="truncate text-xs text-muted-foreground" title={composed}>
          <span className="font-medium">Resolves to:</span>{" "}
          <span className="font-mono">{composed}</span>
        </p>
      )}
    </div>
  );
}
