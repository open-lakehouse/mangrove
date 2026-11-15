import { A2uiSurface } from "@a2ui/react/v0_9";
import type { MessageProcessor, SurfaceModel } from "@a2ui/web_core/v0_9";
import { useEffect, useState } from "react";

/** Every surface `processor` holds, kept in sync as the server creates and
 *  deletes them. */
export function Surfaces({
  processor,
  empty,
}: {
  processor: MessageProcessor;
  empty?: React.ReactNode;
}) {
  const [surfaces, setSurfaces] = useState<SurfaceModel[]>(() =>
    Array.from(processor.model.surfacesMap.values()),
  );
  useEffect(() => {
    const sync = () =>
      setSurfaces(Array.from(processor.model.surfacesMap.values()));
    sync();
    const created = processor.onSurfaceCreated(sync);
    const deleted = processor.onSurfaceDeleted(sync);
    return () => {
      created.unsubscribe();
      deleted.unsubscribe();
    };
  }, [processor]);

  if (surfaces.length === 0) return <>{empty}</>;
  return (
    <>
      {surfaces.map((surface) => (
        <A2uiSurface key={surface.id} surface={surface as never} />
      ))}
    </>
  );
}
