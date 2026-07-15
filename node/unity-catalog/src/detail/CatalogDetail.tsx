import { useCatalogDetail } from "@open-lakehouse/unity-catalog-client";

import { SectionLabel } from "../SectionLabel";
import { DetailStates } from "./DetailStates";
import { formatTimestamp, Meta, MetaGrid } from "./Meta";

export function CatalogDetail({ name }: { name: string }) {
  const { data: catalog, isLoading, error } = useCatalogDetail(name);
  if (!catalog) return <DetailStates isLoading={isLoading} error={error} />;

  // A catalog's storage_location is the auto-generated managed path under
  // storage_root (UUID-laden noise); the storage_root is the meaningful, user-
  // set value. When neither is present the catalog inherits managed storage
  // from the metastore.
  return (
    <section className="space-y-3">
      <SectionLabel>About this catalog</SectionLabel>
      <MetaGrid>
        <Meta label="Owner" value={catalog.owner} />
        <Meta label="Catalog ID" value={catalog.id} mono copyable />
        {catalog.storage_root ? (
          <Meta label="Storage root" value={catalog.storage_root} wide mono />
        ) : (
          <Meta label="Storage" value="Managed by Unity Catalog" wide />
        )}
        <Meta label="Created" value={formatTimestamp(catalog.created_at)} />
        <Meta label="Created by" value={catalog.created_by} />
        <Meta
          label="Last updated"
          value={formatTimestamp(catalog.updated_at)}
        />
        <Meta label="Updated by" value={catalog.updated_by} />
        <Meta label="Comment" value={catalog.comment} wide />
      </MetaGrid>
    </section>
  );
}
