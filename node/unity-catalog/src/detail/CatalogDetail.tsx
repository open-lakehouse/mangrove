import { useCatalogDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function CatalogDetail({ name }: { name: string }) {
  const { data: catalog, isLoading, error } = useCatalogDetail(name);
  if (!catalog) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <MetaGrid>
      <Meta label="Owner" value={catalog.owner} />
      <Meta label="Created by" value={catalog.created_by} />
      <Meta label="Storage root" value={catalog.storage_root} wide mono />
      <Meta
        label="Storage location"
        value={catalog.storage_location}
        wide
        mono
      />
      <Meta label="Comment" value={catalog.comment} wide />
    </MetaGrid>
  );
}
