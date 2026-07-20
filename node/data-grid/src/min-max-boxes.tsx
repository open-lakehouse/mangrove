import { cn } from "@open-lakehouse/ui-kit";
import { useMemo, useState } from "react";
import type { ArrowResultStore } from "./lib/arrowResultStore";
import type { MinMaxAxis } from "./lib/minMaxAxes";
import { type AxisData, useMinMaxBoxes } from "./lib/useMinMaxBoxes";

// The min/max-box visualization for the reconciled-log view. Each surviving file
// carries a [min,max] interval per column (from its stats); this plots those
// intervals so file overlap / data-skipping effectiveness is visible at a glance.
//
// 1D: a horizontal interval per file on one axis (a number line).
// 2D: a bounding-box rectangle per file across two axes.
//
// Files are one population, so a single sequential-blue series at low alpha is
// used (overlap reads via alpha stacking + a thin surface ring) rather than a
// per-file categorical palette. SVG marks (hundreds–low-thousands render fine
// and give free hover hit-testing); past MARK_BUDGET we cap and note it.

/** Above this many marks, SVG hit-testing gets heavy; we cap and log the drop.
 *  A Canvas renderer is the future path (see plan) — not silent truncation. */
const MARK_BUDGET = 4000;

const PAD = { top: 16, right: 16, bottom: 40, left: 64 };

interface MinMaxBoxesProps {
  store: ArrowResultStore;
  version: number;
  /** The 1 or 2 axes to plot (order = [x] or [x, y]). */
  axes: MinMaxAxis[];
  className?: string;
}

export function MinMaxBoxes({
  store,
  version,
  axes,
  className,
}: MinMaxBoxesProps) {
  const data = useMinMaxBoxes(store, version, axes);
  const [width, setWidth] = useState(720);
  const height = 360;
  const [hover, setHover] = useState<number | null>(null);

  // Cap marks to the budget; note how many were dropped (never silent).
  const shown = Math.min(data.count, MARK_BUDGET);
  const dropped = data.count - shown;

  const plot = {
    x: PAD.left,
    y: PAD.top,
    w: Math.max(0, width - PAD.left - PAD.right),
    h: height - PAD.top - PAD.bottom,
  };

  if (axes.length === 0 || data.axes.length === 0) {
    return (
      <EmptyState className={className} message="Select a column to plot." />
    );
  }
  const xData = data.axes[0];
  if (!xData.domain) {
    return (
      <EmptyState
        className={className}
        message={`No min/max stats available for "${xData.axis.name}".`}
      />
    );
  }
  const yData = data.axes[1];
  const is2d = axes.length >= 2 && !!yData;
  if (is2d && !yData.domain) {
    return (
      <EmptyState
        className={className}
        message={`No min/max stats available for "${yData.axis.name}".`}
      />
    );
  }

  const xScale = makeScale(xData.domain, plot.x, plot.x + plot.w);
  // In 1D, files stack along the vertical; in 2D, y maps the second axis.
  const yScale =
    is2d && yData.domain
      ? makeScale(yData.domain, plot.y + plot.h, plot.y)
      : null;

  return (
    <figure
      className={cn(
        "min-h-0 flex-1 overflow-hidden rounded border bg-background",
        className,
      )}
    >
      <Measured onWidth={setWidth}>
        <svg
          width="100%"
          height={height}
          viewBox={`0 0 ${width} ${height}`}
          role="img"
          aria-label={
            is2d
              ? `Per-file min/max boxes: ${xData.axis.name} by ${yData?.axis.name}`
              : `Per-file min/max intervals on ${xData.axis.name}`
          }
        >
          <Axes
            plot={plot}
            xLabel={xData.axis.name}
            yLabel={is2d ? yData?.axis.name : "files"}
          />
          {is2d && yScale
            ? render2d(xData, yData, xScale, yScale, shown, hover, setHover)
            : render1d(xData, plot, xScale, shown, hover, setHover)}
        </svg>
        {hover !== null && (
          <Tooltip
            path={data.paths[hover] || `file #${hover}`}
            axes={data.axes}
            index={hover}
          />
        )}
        {dropped > 0 && (
          <p className="px-3 py-1 text-xs text-muted-foreground">
            Showing {shown.toLocaleString()} of {data.count.toLocaleString()}{" "}
            files ({dropped.toLocaleString()} not drawn — Canvas rendering
            pending).
          </p>
        )}
      </Measured>
    </figure>
  );
}

/** 1D: a horizontal interval per file, rows stacked top→bottom. */
function render1d(
  xData: AxisData,
  plot: { x: number; y: number; w: number; h: number },
  xScale: (v: number) => number,
  shown: number,
  hover: number | null,
  setHover: (i: number | null) => void,
) {
  // Only files with a finite interval occupy a row; lay them out evenly.
  const rows: number[] = [];
  for (let i = 0; i < shown; i++) {
    if (Number.isFinite(xData.min[i]) && Number.isFinite(xData.max[i])) {
      rows.push(i);
    }
  }
  const rowH = rows.length > 0 ? plot.h / rows.length : plot.h;
  const barH = Math.max(2, Math.min(10, rowH - 2));
  return (
    <g>
      {rows.map((i, r) => {
        const x1 = xScale(xData.min[i]);
        const x2 = xScale(xData.max[i]);
        const y = plot.y + r * rowH + (rowH - barH) / 2;
        const active = hover === i;
        return (
          // biome-ignore lint/a11y/noStaticElementInteractions: decorative mark; hover is progressive enhancement over the Grid table-view fallback
          <rect
            key={i}
            x={Math.min(x1, x2)}
            y={y}
            width={Math.max(1, Math.abs(x2 - x1))}
            height={barH}
            rx={2}
            className={cn(
              "fill-primary/45 stroke-background",
              active && "fill-primary stroke-foreground",
            )}
            strokeWidth={active ? 1.5 : 0.75}
            onMouseEnter={() => setHover(i)}
            onMouseLeave={() => setHover(null)}
          />
        );
      })}
    </g>
  );
}

/** 2D: a bounding-box rectangle per file across the two axes. */
function render2d(
  xData: AxisData,
  yData: AxisData,
  xScale: (v: number) => number,
  yScale: (v: number) => number,
  shown: number,
  hover: number | null,
  setHover: (i: number | null) => void,
) {
  const boxes: number[] = [];
  for (let i = 0; i < shown; i++) {
    if (
      Number.isFinite(xData.min[i]) &&
      Number.isFinite(xData.max[i]) &&
      Number.isFinite(yData.min[i]) &&
      Number.isFinite(yData.max[i])
    ) {
      boxes.push(i);
    }
  }
  return (
    <g>
      {boxes.map((i) => {
        const x1 = xScale(xData.min[i]);
        const x2 = xScale(xData.max[i]);
        const y1 = yScale(yData.min[i]);
        const y2 = yScale(yData.max[i]);
        const active = hover === i;
        return (
          // biome-ignore lint/a11y/noStaticElementInteractions: decorative mark; hover is progressive enhancement over the Grid table-view fallback
          <rect
            key={i}
            x={Math.min(x1, x2)}
            y={Math.min(y1, y2)}
            width={Math.max(1, Math.abs(x2 - x1))}
            height={Math.max(1, Math.abs(y2 - y1))}
            className={cn(
              "fill-primary/15 stroke-primary/50",
              active && "fill-primary/30 stroke-foreground",
            )}
            strokeWidth={active ? 1.5 : 0.75}
            onMouseEnter={() => setHover(i)}
            onMouseLeave={() => setHover(null)}
          />
        );
      })}
    </g>
  );
}

/** A linear scale mapping a data domain to a pixel range. */
function makeScale(
  domain: [number, number],
  r0: number,
  r1: number,
): (v: number) => number {
  const [d0, d1] = domain;
  const span = d1 - d0 || 1; // avoid divide-by-zero for a single value
  return (v: number) => r0 + ((v - d0) / span) * (r1 - r0);
}

function Axes({
  plot,
  xLabel,
  yLabel,
}: {
  plot: { x: number; y: number; w: number; h: number };
  xLabel: string;
  yLabel?: string;
}) {
  return (
    <g className="text-muted-foreground">
      {/* Plot frame */}
      <line
        x1={plot.x}
        y1={plot.y + plot.h}
        x2={plot.x + plot.w}
        y2={plot.y + plot.h}
        className="stroke-border"
        strokeWidth={1}
      />
      <line
        x1={plot.x}
        y1={plot.y}
        x2={plot.x}
        y2={plot.y + plot.h}
        className="stroke-border"
        strokeWidth={1}
      />
      <text
        x={plot.x + plot.w / 2}
        y={plot.y + plot.h + 28}
        textAnchor="middle"
        className="fill-muted-foreground text-[11px]"
      >
        {xLabel}
      </text>
      {yLabel && (
        <text
          x={plot.x - 48}
          y={plot.y + plot.h / 2}
          textAnchor="middle"
          transform={`rotate(-90 ${plot.x - 48} ${plot.y + plot.h / 2})`}
          className="fill-muted-foreground text-[11px]"
        >
          {yLabel}
        </text>
      )}
    </g>
  );
}

function Tooltip({
  path,
  axes,
  index,
}: {
  path: string;
  axes: AxisData[];
  index: number;
}) {
  return (
    <div className="pointer-events-none absolute right-3 top-3 max-w-[60%] rounded border bg-popover px-2 py-1.5 font-mono text-xs shadow-sm">
      <div className="truncate font-medium text-foreground">{path}</div>
      {axes.map((a) => (
        <div key={a.axis.name} className="text-muted-foreground">
          <span className="text-muted-foreground/60">{a.axis.name}: </span>
          {fmt(a.min[index])} … {fmt(a.max[index])}
        </div>
      ))}
    </div>
  );
}

function fmt(v: number): string {
  if (!Number.isFinite(v)) return "—";
  return Math.abs(v) >= 1e15 || (v !== 0 && Math.abs(v) < 1e-3)
    ? v.toExponential(2)
    : v.toLocaleString(undefined, { maximumFractionDigits: 3 });
}

function EmptyState({
  message,
  className,
}: {
  message: string;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex min-h-0 flex-1 items-center justify-center rounded border bg-background px-3 py-10 text-center text-sm text-muted-foreground",
        className,
      )}
    >
      {message}
    </div>
  );
}

/** Track the container's width so the SVG viewBox matches its rendered size
 *  (a simple ResizeObserver wrapper — no chart lib). */
function Measured({
  children,
  onWidth,
}: {
  children: React.ReactNode;
  onWidth: (w: number) => void;
}) {
  const ref = useMemo(() => {
    let el: HTMLDivElement | null = null;
    let ro: ResizeObserver | null = null;
    return (node: HTMLDivElement | null) => {
      if (ro) {
        ro.disconnect();
        ro = null;
      }
      el = node;
      if (el && typeof ResizeObserver !== "undefined") {
        ro = new ResizeObserver((entries) => {
          const w = entries[0]?.contentRect.width;
          if (w) onWidth(Math.round(w));
        });
        ro.observe(el);
      }
    };
  }, [onWidth]);
  return (
    <div ref={ref} className="relative">
      {children}
    </div>
  );
}
