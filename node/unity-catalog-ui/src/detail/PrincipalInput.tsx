import { Input } from "@open-lakehouse/ui-kit";
import type { PrincipalRef } from "@open-lakehouse/unity-catalog-client";
import { useIdentityHost } from "@open-lakehouse/unity-catalog-client/react";
import { useEffect, useId, useState } from "react";

const SEARCH_DEBOUNCE_MS = 250;
const SEARCH_LIMIT = 8;

const KIND_LABEL: Record<PrincipalRef["kind"], string> = {
  user: "User",
  group: "Group",
  service_account: "Service account",
};

/**
 * A principal name field. With an identity directory in scope (the host's
 * `identity`, or an `<IdentityProvider>`), it suggests matching users, groups
 * and service accounts as you type; without one it is a plain text field and
 * the server validates the name on submit.
 */
export function PrincipalInput({
  value,
  onChange,
  id,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  id?: string;
  disabled?: boolean;
}) {
  const identity = useIdentityHost();
  const listId = useId();
  const [results, setResults] = useState<PrincipalRef[]>([]);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const query = value.trim();
    if (!identity || !query) {
      setResults([]);
      return;
    }
    // A newer keystroke aborts the older search, so results never arrive out
    // of order.
    const controller = new AbortController();
    const timer = setTimeout(() => {
      identity
        .search(query, { limit: SEARCH_LIMIT, signal: controller.signal })
        .then((found) => {
          if (!controller.signal.aborted) setResults(found);
        })
        .catch(() => {
          if (!controller.signal.aborted) setResults([]);
        });
    }, SEARCH_DEBOUNCE_MS);
    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [identity, value]);

  function pick(name: string) {
    onChange(name);
    setOpen(false);
  }

  const suggestions = results.filter((r) => r.name !== value);
  const showList = open && suggestions.length > 0;

  return (
    <div className="relative">
      <Input
        id={id}
        value={value}
        disabled={disabled}
        autoComplete="off"
        placeholder={
          identity
            ? "Search users, groups, service accounts"
            : "user@example.com or group name"
        }
        aria-controls={showList ? listId : undefined}
        onChange={(e) => {
          onChange(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        // Delay so a click on a suggestion lands before the list closes.
        onBlur={() => setTimeout(() => setOpen(false), 100)}
        onKeyDown={(e) => {
          if (e.key === "Escape") setOpen(false);
        }}
      />
      {showList && (
        <ul
          id={listId}
          aria-label="Matching principals"
          className="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border bg-popover p-1 text-sm shadow-md"
        >
          {suggestions.map((principal) => (
            <li key={`${principal.kind}:${principal.name}`}>
              <button
                type="button"
                className="flex w-full items-center justify-between gap-3 rounded-sm px-2 py-1.5 text-left hover:bg-accent focus:bg-accent focus:outline-none"
                // mousedown, not click: it lands before the input's blur closes
                // the list.
                onMouseDown={(e) => {
                  e.preventDefault();
                  pick(principal.name);
                }}
                onClick={() => pick(principal.name)}
              >
                <span className="min-w-0 truncate">
                  {principal.displayName ? (
                    <>
                      {principal.displayName}{" "}
                      <span className="text-muted-foreground">
                        {principal.name}
                      </span>
                    </>
                  ) : (
                    principal.name
                  )}
                </span>
                <span className="shrink-0 text-xs text-muted-foreground">
                  {KIND_LABEL[principal.kind]}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
