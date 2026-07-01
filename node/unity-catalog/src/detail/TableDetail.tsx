import { Columns3 } from "lucide-react";

import { useTableDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function TableDetail({ fullName }: { fullName: string }) {
  const { data: table, isLoading, error } = useTableDetail(fullName);
  if (!table) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <>
      <MetaGrid>
        <Meta label="Owner" value={table.owner} />
        <Meta label="Type" value={table.table_type} />
        <Meta label="Format" value={table.data_source_format} />
        <Meta
          label="Storage location"
          value={table.storage_location}
          wide
          mono
        />
        <Meta label="Comment" value={table.comment} wide />
      </MetaGrid>

      <div className="mt-6">
        <div className="mb-2 flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          <Columns3 className="h-4 w-4" />
          Columns
        </div>
        {table.columns && table.columns.length > 0 ? (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b text-left text-xs text-muted-foreground">
                <th className="py-1 pr-4 font-medium">Name</th>
                <th className="py-1 pr-4 font-medium">Type</th>
                <th className="py-1 font-medium">Comment</th>
              </tr>
            </thead>
            <tbody>
              {table.columns.map((col) => (
                <tr key={col.name} className="border-b last:border-b-0">
                  <td className="py-1 pr-4 font-mono">{col.name}</td>
                  <td className="py-1 pr-4 text-muted-foreground">
                    {col.type_text ?? "—"}
                  </td>
                  <td className="py-1 text-muted-foreground">
                    {col.comment ?? ""}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="text-sm text-muted-foreground">No column metadata.</p>
        )}
      </div>
    </>
  );
}
