import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsList,
  TabsTrigger,
} from "@open-lakehouse/ui-kit";
import type { MinMaxAxis } from "./lib/minMaxAxes";

// Axis controls for the min/max-box view: a 1D/2D dimension toggle and one
// column Select per active axis. The available options are the enumerated
// orderable columns (those with both min and max present).

export type BoxDim = "1d" | "2d";

interface MinMaxControlsProps {
  available: MinMaxAxis[];
  dim: BoxDim;
  onDim: (d: BoxDim) => void;
  /** Selected axis names: [x] in 1D, [x, y] in 2D. */
  selected: string[];
  onSelect: (index: number, name: string) => void;
}

export function MinMaxControls({
  available,
  dim,
  onDim,
  selected,
  onSelect,
}: MinMaxControlsProps) {
  return (
    <div className="mb-2 flex flex-wrap items-center gap-2">
      <Tabs value={dim} onValueChange={(v) => onDim(v as BoxDim)}>
        <TabsList>
          <TabsTrigger value="1d">1D</TabsTrigger>
          <TabsTrigger value="2d">2D</TabsTrigger>
        </TabsList>
      </Tabs>

      <AxisSelect
        label="x"
        value={selected[0]}
        options={available}
        onChange={(name) => onSelect(0, name)}
      />
      {dim === "2d" && (
        <AxisSelect
          label="y"
          value={selected[1]}
          options={available}
          onChange={(name) => onSelect(1, name)}
        />
      )}
    </div>
  );
}

function AxisSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string | undefined;
  options: MinMaxAxis[];
  onChange: (name: string) => void;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-xs font-medium uppercase text-muted-foreground">
        {label}
      </span>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="h-8 w-40 text-xs">
          <SelectValue placeholder="Select column" />
        </SelectTrigger>
        <SelectContent>
          {options.map((a) => (
            <SelectItem key={a.name} value={a.name} className="text-xs">
              {a.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
