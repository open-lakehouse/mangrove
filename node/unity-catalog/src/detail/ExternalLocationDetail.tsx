import { useExternalLocationDetail } from "@open-lakehouse/unity-catalog-client";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function ExternalLocationDetail({ name }: { name: string }) {
  const { data: location, isLoading, error } = useExternalLocationDetail(name);
  if (!location) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <MetaGrid>
      <Meta label="Credential" value={location.credential_name} />
      <Meta label="Owner" value={location.owner} />
      <Meta label="Created by" value={location.created_by} />
      <Meta label="URL" value={location.url} wide mono />
      <Meta label="Comment" value={location.comment} wide />
    </MetaGrid>
  );
}
