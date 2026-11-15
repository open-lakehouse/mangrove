import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import type { GrantSecurableType } from "@open-lakehouse/unity-catalog-client";
import { type ReactNode, useState } from "react";

import { PermissionsTab } from "./PermissionsTab.js";

/**
 * The Overview / Permissions tab pair for a securable whose detail has no tabs
 * of its own. The tab is local state, not the URL `tab` param (the schema view
 * owns that for its child-kind filter).
 */
export function OverviewTabs({
  securableType,
  fullName,
  children,
}: {
  securableType: GrantSecurableType;
  fullName: string;
  children: ReactNode;
}) {
  const [view, setView] = useState<"overview" | "permissions">("overview");
  return (
    <Tabs
      value={view}
      onValueChange={(v) => setView(v as "overview" | "permissions")}
    >
      <TabsList>
        <TabsTrigger value="overview">Overview</TabsTrigger>
        <TabsTrigger value="permissions">Permissions</TabsTrigger>
      </TabsList>
      <TabsContent value="overview">{children}</TabsContent>
      <TabsContent value="permissions">
        <PermissionsTab securableType={securableType} fullName={fullName} />
      </TabsContent>
    </Tabs>
  );
}
