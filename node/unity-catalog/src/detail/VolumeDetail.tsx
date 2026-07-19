import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import { useVolumeDetail } from "@open-lakehouse/unity-catalog-client";
import { useState } from "react";

import { SectionLabel } from "../SectionLabel";
import { DetailStates } from "./DetailStates";
import { formatTimestamp, Meta, MetaGrid } from "./Meta";
import { TypePill } from "./TypePill";
import { VolumeEditor } from "./VolumeEditor";

type VolumeView = "overview" | "files";

// The volume file editor is additive and dark-launched behind a flag (mirrors
// the preview/delta-log gating on TableDetail). Vite statically replaces
// import.meta.env.*; undefined/"false" → off.
const FILES_ENABLED = import.meta.env.VITE_ENABLE_VOLUME_FILES === "true";

export function VolumeDetail({ fullName }: { fullName: string }) {
  const { data: volume, isLoading, error } = useVolumeDetail(fullName);
  const [view, setView] = useState<VolumeView>("overview");
  if (!volume) return <DetailStates isLoading={isLoading} error={error} />;

  // A managed volume's storage_location is a UC-internal path under the
  // metastore root (a long UUID-laden URI that's noise to the user). We surface
  // where the bytes live only for external volumes, where the location is the
  // whole point.
  const managed = volume.volume_type === "MANAGED";

  const overview = (
    <section className="space-y-3">
      <SectionLabel>About this volume</SectionLabel>
      <MetaGrid>
        <Meta label="Type">
          <TypePill value={volume.volume_type} />
        </Meta>
        <Meta label="Owner" value={volume.owner} />
        <Meta label="Volume ID" value={volume.volume_id} mono copyable />
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

  // Without the flag there's only metadata — render it directly (no lone tab).
  if (!FILES_ENABLED) return overview;

  return (
    <Tabs value={view} onValueChange={(v) => setView(v as VolumeView)}>
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="files">Files</TabsTrigger>
      </TabsList>
      <TabsContent value="overview">{overview}</TabsContent>
      <TabsContent value="files">
        <VolumeEditor fullName={fullName} />
      </TabsContent>
    </Tabs>
  );
}
