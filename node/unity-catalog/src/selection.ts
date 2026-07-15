// URL-addressable selection for the Catalog Explorer.
//
// The selected object lives in the route's `sel` search param (encoded as
// `kind:fullName`) instead of component state, so a detail view is deep-linkable
// and survives reloads / back-forward navigation. Everything reads/writes it
// through this hook so the encoding stays in one place.
import { useNavigate, useSearch } from "@tanstack/react-router";

import {
  OBJECT_KINDS,
  type ObjectKind,
  SELECTABLE_KINDS,
  type SelectableKind,
  type Selection,
} from "./types";

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

/** Validate the raw `tab` search param down to a known object kind. */
function decodeSchemaTab(raw: string | undefined): ObjectKind | undefined {
  return OBJECT_KINDS.includes(raw as ObjectKind)
    ? (raw as ObjectKind)
    : undefined;
}

export function useCatalogSelection() {
  const raw = useSearch({ from: FROM, select: (s) => s.sel });
  // Which child-kind tab a selected schema opens on. Lives alongside `sel` so a
  // schema view is deep-linkable to a specific kind (Tables/Volumes/...), and so
  // the tree's kind rows and SchemaDetail's filter bar stay in sync.
  const rawTab = useSearch({ from: FROM, select: (s) => s.tab });
  const navigate = useNavigate({ from: FROM });
  const selection = decodeSelection(raw);
  const schemaTab = decodeSchemaTab(rawTab);

  // `prev` is the route's full search object; typed via the app's router
  // registration (annotated here so the package also type-checks standalone,
  // where no route is registered).
  function select(next: Selection | undefined) {
    navigate({
      // A normal selection clears any schema tab: the tab only scopes a schema
      // view, and a fresh selection should open schemas on their default kind.
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        sel: next ? encodeSelection(next) : undefined,
        tab: undefined,
      }),
      replace: true,
    });
  }

  /** Select a schema and open it on a specific child-kind tab (tree kind rows). */
  function selectSchemaChild(schemaFullName: string, kind: ObjectKind) {
    navigate({
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        sel: encodeSelection({ kind: "schema", fullName: schemaFullName }),
        tab: kind,
      }),
      replace: true,
    });
  }

  /** Change only the active schema tab (SchemaDetail's filter bar). */
  function setSchemaTab(kind: ObjectKind) {
    navigate({
      search: (prev: Record<string, unknown>) => ({ ...prev, tab: kind }),
      replace: true,
    });
  }

  return {
    selection,
    schemaTab,
    select,
    selectSchemaChild,
    setSchemaTab,
  } as const;
}
