// URL-addressable selection for the Catalog Explorer.
//
// The selected object lives in the route's `sel` search param (encoded as
// `kind:fullName`) instead of component state, so a detail view is deep-linkable
// and survives reloads / back-forward navigation. Everything reads/writes it
// through this hook so the encoding stays in one place.
import { useNavigate, useSearch } from "@tanstack/react-router";

import { SELECTABLE_KINDS, type SelectableKind, type Selection } from "./types";

/** Route id the selection lives on (see routeTree.tsx `validateSearch`). */
const FROM = "/catalog";

export function encodeSelection(selection: Selection): string {
  return `${selection.kind}:${selection.fullName}`;
}

export function decodeSelection(
  raw: string | undefined,
): Selection | undefined {
  if (!raw) return undefined;
  const sep = raw.indexOf(":");
  if (sep < 0) return undefined;
  const kind = raw.slice(0, sep) as SelectableKind;
  const fullName = raw.slice(sep + 1);
  if (!fullName || !SELECTABLE_KINDS.includes(kind)) return undefined;
  return { kind, fullName };
}

export function useCatalogSelection() {
  const raw = useSearch({ from: FROM, select: (s) => s.sel });
  const navigate = useNavigate({ from: FROM });
  const selection = decodeSelection(raw);

  function select(next: Selection | undefined) {
    navigate({
      // `prev` is the route's full search object; typed via the app's router
      // registration (annotated here so the package also type-checks standalone,
      // where no route is registered).
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        sel: next ? encodeSelection(next) : undefined,
      }),
      replace: true,
    });
  }

  return { selection, select } as const;
}
