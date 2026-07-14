import { Badge } from "@open-lakehouse/ui-kit";
import { useTableDetail } from "@open-lakehouse/unity-catalog-client";
import { Columns3 } from "lucide-react";

import { DetailStates } from "./DetailStates";
import { FormatIcon } from "./FormatIcon";
import { TablePreview } from "./TablePreview";

// Header adornments for a table: its data-source-format icon + table-type pill,
// rendered by DetailPane to the right of the name. Reads the same cached query
// as the body (react-query dedupes by key), so mounting it here is free.
export function TableHeaderMeta({ fullName }: { fullName: string }) {
  const { data: table } = useTableDetail(fullName);
  if (!table) return null;
  return (
    <>
      {table.data_source_format && (
        <FormatIcon
          format={table.data_source_format}
          className="h-5 w-5 shrink-0"
        />
      )}
      {table.table_type && (
        <Badge variant="primary" className="shrink-0">
          {table.table_type}
        </Badge>
      )}
    </>
  );
}

export function TableDetail({ fullName }: { fullName: string }) {
  const { data: table, isLoading, error } = useTableDetail(fullName);
  if (!table) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <>
      {(table.owner || table.comment) && (
        <div className="space-y-1">
          {table.owner && (
            <p className="text-sm text-muted-foreground">
              Owned by <span className="text-foreground">{table.owner}</span>
            </p>
          )}
          {table.comment && (
            <p className="max-w-prose text-sm text-muted-foreground">
              {table.comment}
            </p>
          )}
        </div>
      )}

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

      <TablePreview
        fullName={fullName}
        format={table.data_source_format}
        tableType={table.table_type}
      />
    </>
  );
}
