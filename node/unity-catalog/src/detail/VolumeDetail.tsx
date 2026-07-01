import { useVolumeDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function VolumeDetail({ fullName }: { fullName: string }) {
  const { data: volume, isLoading, error } = useVolumeDetail(fullName);
  if (!volume) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <MetaGrid>
      <Meta label="Owner" value={volume.owner} />
      <Meta label="Volume type" value={volume.volume_type} />
      <Meta
        label="Storage location"
        value={volume.storage_location}
        wide
        mono
      />
      <Meta label="Comment" value={volume.comment} wide />
    </MetaGrid>
  );
}
