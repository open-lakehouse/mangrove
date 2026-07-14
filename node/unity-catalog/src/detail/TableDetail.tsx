import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import { useTableDetail } from "@open-lakehouse/unity-catalog-client";
import { useState } from "react";

import { SectionLabel } from "../SectionLabel";
import { DetailStates } from "./DetailStates";
import { FormatIcon } from "./FormatIcon";
import { formatTimestamp, Meta, MetaGrid } from "./Meta";
import { TablePreview } from "./TablePreview";
import { TypePill } from "./TypePill";

export function TableDetail({ fullName }: { fullName: string }) {
  const { data: table, isLoading, error } = useTableDetail(fullName);
  const [view, setView] = useState<"overview" | "details">("overview");
  if (!table) return <DetailStates isLoading={isLoading} error={error} />;

  // A managed table's storage_location is a UC-internal path under the
  // metastore root (a long UUID-laden URI that's noise to the user). We surface
  // where the bytes live only for external tables, where it's meaningful.
  const managed = table.table_type === "MANAGED";

  return (
    <Tabs
      value={view}
      onValueChange={(v) => setView(v as "overview" | "details")}
    >
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="details">Details</TabsTrigger>
      </TabsList>

      <TabsContent value="overview" className="space-y-6">
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

        <div>
          <SectionLabel className="mb-2">Columns</SectionLabel>
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
      </TabsContent>

      <TabsContent value="details">
        <section className="space-y-3">
          <SectionLabel>About this table</SectionLabel>
          <MetaGrid>
            <Meta label="Type">
              <TypePill value={table.table_type} />
            </Meta>
            <Meta label="Owner" value={table.owner} />
            <Meta label="Table ID" value={table.table_id} mono copyable />
            <Meta label="Data source format">
              {table.data_source_format ? (
                <span className="flex min-w-0 items-center gap-1.5">
                  <FormatIcon
                    format={table.data_source_format}
                    className="h-4 w-4 shrink-0"
                  />
                  <span className="truncate">{table.data_source_format}</span>
                </span>
              ) : (
                <span className="text-muted-foreground">—</span>
              )}
            </Meta>
            {managed ? (
              <Meta
                label="Storage location"
                value="Managed by Unity Catalog"
                wide
              />
            ) : (
              <Meta
                label="Storage location"
                value={table.storage_location}
                wide
                mono
              />
            )}
            <Meta label="Created" value={formatTimestamp(table.created_at)} />
            <Meta label="Created by" value={table.created_by} />
            <Meta
              label="Last updated"
              value={formatTimestamp(table.updated_at)}
            />
            <Meta label="Updated by" value={table.updated_by} />
            <Meta label="Comment" value={table.comment} wide />
          </MetaGrid>
        </section>
      </TabsContent>
    </Tabs>
  );
}
