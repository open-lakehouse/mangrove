// Storage-location picker for catalog/schema creation.
//
// Mirrors the database behavior: leaving it "Managed" lets the securable
// inherit its parent's managed storage. Toggling managed off requires an
// explicit storage root — either a defined external location (optionally with a
// subpath under it) or a custom storage URL entered directly (useful before any
// external locations are defined, or for ad-hoc roots).

import {
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from "@open-lakehouse/ui-kit";
import { useExternalLocations } from "@open-lakehouse/unity-catalog-client";
import { useId, useMemo, useState } from "react";

// Radix Select can't use an empty-string value, so the custom mode needs a sentinel.
const CUSTOM = "__custom__";

export function StorageLocationPicker({
  onChange,
}: {
  /** Called with the composed storage location, or undefined for managed storage. */
  onChange: (storageLocation: string | undefined) => void;
}) {
  const locations = useExternalLocations();
  const [managed, setManaged] = useState(true);
  const [source, setSource] = useState(CUSTOM);
  const [subpath, setSubpath] = useState("");
  const [customUrl, setCustomUrl] = useState("");

  const managedId = useId();
  const sourceId = useId();
  const subpathId = useId();
  const urlId = useId();

  const items = useMemo(
    () => (locations.data ?? []).filter((l) => !!l.name && !!l.url),
    [locations.data],
  );

  function compose(
    isManaged: boolean,
    nextSource: string,
    sub: string,
    custom: string,
  ) {
    if (isManaged) return undefined;
    if (nextSource === CUSTOM) return custom.trim() || undefined;
    const base = items.find((l) => l.name === nextSource)?.url;
    if (!base) return undefined;
    const trimmedBase = base.replace(/\/+$/, "");
    const trimmedSub = sub.trim().replace(/^\/+/, "");
    return trimmedSub ? `${trimmedBase}/${trimmedSub}` : trimmedBase;
  }

  function toggleManaged(next: boolean) {
    setManaged(next);
    onChange(compose(next, source, subpath, customUrl));
  }

  function selectSource(next: string) {
    setSource(next);
    onChange(compose(managed, next, subpath, customUrl));
  }

  function changeSubpath(sub: string) {
    setSubpath(sub);
    onChange(compose(managed, source, sub, customUrl));
  }

  function changeCustomUrl(url: string) {
    setCustomUrl(url);
    onChange(compose(managed, source, subpath, url));
  }

  const composed = compose(managed, source, subpath, customUrl);
  const isLocation = source !== CUSTOM;

  return (
    <div className="space-y-2 rounded-md border bg-muted/30 p-3">
      <div className="flex items-center justify-between gap-3">
        <div className="space-y-0.5">
          <Label htmlFor={managedId}>Managed storage</Label>
          <p className="text-xs text-muted-foreground">
            Inherit managed storage from the parent.
          </p>
        </div>
        <Switch
          id={managedId}
          checked={managed}
          onCheckedChange={toggleManaged}
        />
      </div>

      {!managed && (
        <div className="space-y-2 border-t pt-2">
          <div className="space-y-1.5">
            <Label htmlFor={sourceId}>Storage location</Label>
            <Select value={source} onValueChange={selectSource}>
              <SelectTrigger id={sourceId}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
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
              Choose a defined external location or enter a storage URL.
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

          {source === CUSTOM && (
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
