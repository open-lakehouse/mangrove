// After a successful create, reveal the new object in the tree using the
// existing expansion + selection stores (no new state channel).

import type { CreateRequest } from "./dialog-types";
import { nodeId, useExpansion } from "./ExpansionContext";
import { useCatalogSelection } from "./selection";
import type { ObjectKind, Selection } from "./types";

export interface CreatedEntity {
  kind: CreateRequest["kind"];
  name: string;
  catalog?: string;
  schema?: string;
}

function revealPlan(created: CreatedEntity): {
  expandIds: string[];
  selection?: Selection;
  schemaTab?: ObjectKind;
} {
  const { kind, name, catalog, schema } = created;

  if (kind === "catalog") {
    return {
      expandIds: [],
      selection: { kind: "catalog", fullName: name },
    };
  }

  if (kind === "schema" && catalog) {
    return {
      expandIds: [nodeId.catalog(catalog)],
      selection: { kind: "schema", fullName: `${catalog}.${name}` },
    };
  }

  if ((kind === "volume" || kind === "model") && catalog && schema) {
    const fullName = `${catalog}.${schema}.${name}`;
    return {
      expandIds: [
        nodeId.catalog(catalog),
        nodeId.schema(catalog, schema),
        nodeId.group(catalog, schema, kind),
      ],
      selection: { kind, fullName },
    };
  }

  return { expandIds: [] };
}

/** Hook that expands ancestors and selects a freshly created entity. */
export function useRevealCreated() {
  const { expand } = useExpansion();
  const { select } = useCatalogSelection();

  return (created: CreatedEntity) => {
    const plan = revealPlan(created);
    if (plan.expandIds.length) expand(plan.expandIds);
    if (plan.selection) select(plan.selection);
  };
}
