import { useEffect } from "react";

import { CatalogTree } from "./CatalogTree";
import { DetailPane } from "./DetailPane";
import { CatalogDialogsProvider } from "./dialogs";
import { ExpansionProvider, nodeId, useExpansion } from "./ExpansionContext";
import { useCatalogSelection } from "./selection";
import { isObjectKind } from "./types";

export function CatalogExplorer() {
  return (
    <ExpansionProvider>
      <CatalogDialogsProvider>
        <ExplorerLayout />
      </CatalogDialogsProvider>
    </ExpansionProvider>
  );
}

function ExplorerLayout() {
  useExpandToSelection();

  return (
    <div className="flex h-[calc(100vh-3rem)] flex-col">
      <div className="grid min-h-0 flex-1 grid-cols-1 overflow-hidden md:grid-cols-[minmax(18rem,24rem)_minmax(0,1fr)]">
        <div className="flex min-h-0 flex-col border-r bg-sidebar">
          <CatalogTree />
        </div>
        <DetailPane />
      </div>
    </div>
  );
}

/**
 * Expand the *ancestors* of the selected node so a deep-linked object becomes
 * visible. We never expand the selected node itself — selecting a catalog or
 * schema must not toggle it (that's the chevron's job).
 */
function useExpandToSelection() {
  const { selection } = useCatalogSelection();
  const { expand } = useExpansion();

  // Depend on the primitive selection fields, NOT the `selection` object:
  // it is re-created on every render (decodeSelection), so using it as a dep
  // would re-run this effect on every render and re-expand ancestors — which
  // would fight the user trying to collapse a parent while a child is selected.
  const kind = selection?.kind;
  const fullName = selection?.fullName;

  useEffect(() => {
    if (!kind || !fullName) return;
    const [catalog, schema] = fullName.split(".");
    const ids: string[] = [];
    // A schema (or deeper) needs its catalog open; a leaf also needs its schema
    // and the matching group open.
    if (catalog && schema) ids.push(nodeId.catalog(catalog));
    if (catalog && schema && isObjectKind(kind)) {
      ids.push(nodeId.schema(catalog, schema));
      ids.push(nodeId.group(catalog, schema, kind));
    }
    if (ids.length) expand(ids);
  }, [kind, fullName, expand]);
}
