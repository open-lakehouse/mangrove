import type {
  FunctionInfo,
  RegisteredModelInfo,
  TableInfo,
  VolumeInfo,
} from "@open-lakehouse/unity-catalog-client";
import {
  useFunctions,
  useModels,
  useTables,
  useVolumes,
} from "@open-lakehouse/unity-catalog-client";
import type { UseInfiniteQueryResult } from "@tanstack/react-query";
import {
  Boxes,
  FunctionSquare,
  HardDrive,
  type LucideIcon,
  Table2,
} from "lucide-react";
import type { ReactNode } from "react";
import type { ObjectKind } from "./types";

export interface GroupDef {
  kind: ObjectKind;
  title: string;
  Icon: LucideIcon;
  // Only the kinds with a low-complexity create form expose an inline "+".
  creatable?: "volume" | "model";
  useList: (
    catalog: string | undefined,
    schema: string | undefined,
  ) => UseInfiniteQueryResult<
    (TableInfo | VolumeInfo | FunctionInfo | RegisteredModelInfo)[],
    unknown
  >;
}

export const GROUPS: GroupDef[] = [
  {
    kind: "table",
    title: "Tables",
    Icon: Table2,
    useList: useTables as GroupDef["useList"],
  },
  {
    kind: "volume",
    title: "Volumes",
    Icon: HardDrive,
    creatable: "volume",
    useList: useVolumes as GroupDef["useList"],
  },
  {
    kind: "function",
    title: "Functions",
    Icon: FunctionSquare,
    useList: useFunctions as GroupDef["useList"],
  },
  {
    kind: "model",
    title: "Models",
    Icon: Boxes,
    useList: useModels as GroupDef["useList"],
  },
];

const ICONS: Record<ObjectKind, LucideIcon> = {
  table: Table2,
  volume: HardDrive,
  function: FunctionSquare,
  model: Boxes,
};

export function kindIcon(kind: ObjectKind, className?: string): ReactNode {
  const Icon = ICONS[kind];
  return <Icon className={className ?? "h-4 w-4"} />;
}
