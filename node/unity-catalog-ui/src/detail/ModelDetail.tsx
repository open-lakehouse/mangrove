import { useModelDetail } from "@open-lakehouse/unity-catalog-client/react";

import { DetailStates } from "./DetailStates.js";
import { Meta, MetaGrid } from "./Meta.js";

export function ModelDetail({ fullName }: { fullName: string }) {
  const { data: model, isLoading, error } = useModelDetail(fullName);
  if (!model) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <MetaGrid>
      <Meta label="Owner" value={model.owner} />
      <Meta label="Storage location" value={model.storage_location} wide mono />
      <Meta label="Comment" value={model.comment} wide />
    </MetaGrid>
  );
}
