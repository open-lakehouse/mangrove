import { useFunctionDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function FunctionDetail({ fullName }: { fullName: string }) {
  const { data: fn, isLoading, error } = useFunctionDetail(fullName);
  if (!fn) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <>
      <MetaGrid>
        <Meta label="Owner" value={fn.owner} />
        <Meta label="Return type" value={fn.full_data_type ?? fn.data_type} />
        <Meta label="Routine body" value={fn.routine_body} />
        <Meta label="SQL data access" value={fn.sql_data_access} />
        <Meta label="Comment" value={fn.comment} />
      </MetaGrid>
      {fn.routine_definition && (
        <div className="mt-6">
          <div className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Definition
          </div>
          <pre className="overflow-auto rounded border bg-muted p-3 font-mono text-xs">
            {fn.routine_definition}
          </pre>
        </div>
      )}
    </>
  );
}
