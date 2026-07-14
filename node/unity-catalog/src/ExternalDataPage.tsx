// The metastore-level "External Data" admin page: a tabbed browser over the two
// storage securables (external locations + credentials), each rendered by the
// shared StorageTable. Controlled: the active `kind` and its setter come from
// the host route (URL-addressable there), so the page itself stays router-
// agnostic. Wraps CatalogDialogsProvider so the tables' create/edit/delete flows
// work without the catalog explorer being mounted.

import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import {
  useCredentials,
  useExternalLocations,
} from "@open-lakehouse/unity-catalog-client";
import type { UseInfiniteQueryResult } from "@tanstack/react-query";
import { Globe, KeyRound, type LucideIcon } from "lucide-react";

import { CatalogDialogsProvider } from "./dialogs";
import { StorageTable } from "./storage/StorageTable";
import { STORAGE_KINDS, type StorageKind } from "./types";

type StorageList = UseInfiniteQueryResult<unknown[], unknown>;

interface TabDef {
  kind: StorageKind;
  label: string;
  Icon: LucideIcon;
  useList: () => StorageList;
}

const TABS: TabDef[] = [
  {
    kind: "external_location",
    label: "External locations",
    Icon: Globe,
    useList: useExternalLocations as () => StorageList,
  },
  {
    kind: "credential",
    label: "Credentials",
    Icon: KeyRound,
    useList: useCredentials as () => StorageList,
  },
];

export function ExternalDataPage({
  kind,
  onKindChange,
}: {
  kind: StorageKind;
  onKindChange: (kind: StorageKind) => void;
}) {
  // Guard against an unknown `kind` from the URL so a bad deep link still lands
  // on a valid tab rather than a blank table.
  const active = STORAGE_KINDS.includes(kind) ? kind : STORAGE_KINDS[0];

  return (
    <CatalogDialogsProvider>
      <div className="flex h-full min-h-0 flex-col">
        <div className="shrink-0 px-6 pt-4">
          <h1 className="font-mono text-base font-semibold">External Data</h1>
        </div>

        <Tabs
          value={active}
          onValueChange={(v) => onKindChange(v as StorageKind)}
          className="flex min-h-0 flex-1 flex-col"
        >
          <div className="shrink-0 px-6 pt-3">
            <TabsList>
              {TABS.map((t) => (
                <TabsTrigger key={t.kind} value={t.kind} className="gap-1.5">
                  <t.Icon className="h-4 w-4" />
                  {t.label}
                  <TabCount useList={t.useList} />
                </TabsTrigger>
              ))}
            </TabsList>
          </div>

          {TABS.map((t) => (
            <TabsContent
              key={t.kind}
              value={t.kind}
              className="mt-0 flex min-h-0 flex-1 flex-col"
            >
              <StorageTable kind={t.kind} />
            </TabsContent>
          ))}
        </Tabs>
      </div>
    </CatalogDialogsProvider>
  );
}

// Live count shown on each tab, read from the same list query the table uses
// (react-query dedupes by key, so this adds no extra request). A trailing "+"
// marks an unfetched next page.
function TabCount({ useList }: { useList: () => StorageList }) {
  const { data, hasNextPage } = useList();
  if (data === undefined) return null;
  return (
    <span className="font-mono text-xs tabular-nums text-muted-foreground">
      {data.length}
      {hasNextPage ? "+" : ""}
    </span>
  );
}
