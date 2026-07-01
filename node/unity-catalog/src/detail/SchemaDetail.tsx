import { useSchemaDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function SchemaDetail({ fullName }: { fullName: string }) {
  const { data: schema, isLoading, error } = useSchemaDetail(fullName);
  if (!schema) return <DetailStates isLoading={isLoading} error={error} />;

  // The server only returns a schema's storage fields when it was created with
  // an explicit location; an inherited schema reports neither and resolves its
  // managed storage from the parent catalog at write time.
  const inheritsStorage = !schema.storage_root && !schema.storage_location;

  return (
    <MetaGrid>
      <Meta label="Owner" value={schema.owner} />
      <Meta label="Catalog" value={schema.catalog_name} />
      <Meta label="Storage root" value={schema.storage_root} wide mono />
      {inheritsStorage ? (
        <Meta
          label="Storage location"
          value="Inherited from parent catalog"
          wide
        />
      ) : (
        <Meta
          label="Storage location"
          value={schema.storage_location}
          wide
          mono
        />
      )}
      <Meta label="Comment" value={schema.comment} wide />
    </MetaGrid>
  );
}
