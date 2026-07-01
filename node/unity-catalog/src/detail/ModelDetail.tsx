import { useModelDetail } from "@open-lakehouse/unity-catalog-client";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

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
