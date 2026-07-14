import { Badge } from "@open-lakehouse/ui-kit";
import { useVolumeDetail } from "@open-lakehouse/unity-catalog-client";

import { SectionLabel } from "../SectionLabel";
import { DetailStates } from "./DetailStates";
import { formatTimestamp, Meta, MetaGrid } from "./Meta";

// Header adornment: the volume-type pill (MANAGED / EXTERNAL), rendered by
// DetailPane to the right of the name. Reads the same cached query as the body
// (react-query dedupes by key), so mounting it here is free.
export function VolumeHeaderMeta({ fullName }: { fullName: string }) {
  const { data: volume } = useVolumeDetail(fullName);
  if (!volume?.volume_type) return null;
  return (
    <Badge
      variant={volume.volume_type === "MANAGED" ? "success" : "primary"}
      className="shrink-0"
    >
      {volume.volume_type}
    </Badge>
  );
}

export function VolumeDetail({ fullName }: { fullName: string }) {
  const { data: volume, isLoading, error } = useVolumeDetail(fullName);
  if (!volume) return <DetailStates isLoading={isLoading} error={error} />;

  // A managed volume's storage_location is a UC-internal path under the
  // metastore root (a long UUID-laden URI that's noise to the user). We surface
  // where the bytes live only for external volumes, where the location is the
  // whole point.
  const managed = volume.volume_type === "MANAGED";

  return (
    <section className="space-y-3">
      <SectionLabel>About this volume</SectionLabel>
      <MetaGrid>
        <Meta label="Owner" value={volume.owner} />
        <Meta label="Volume ID" value={volume.volume_id} mono />
        {managed ? (
          <Meta
            label="Storage location"
            value="Managed by Unity Catalog"
            wide
          />
        ) : (
          <Meta
            label="Storage location"
            value={volume.storage_location}
            wide
            mono
          />
        )}
        <Meta label="Created" value={formatTimestamp(volume.created_at)} />
        <Meta label="Created by" value={volume.created_by} />
        <Meta label="Last updated" value={formatTimestamp(volume.updated_at)} />
        <Meta label="Updated by" value={volume.updated_by} />
        <Meta label="Comment" value={volume.comment} wide />
      </MetaGrid>
    </section>
  );
}
