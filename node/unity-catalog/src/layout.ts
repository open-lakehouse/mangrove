// Shared chrome for the explorer's two column headers — the sidebar tree header
// (CatalogTree) and the detail-pane header (DetailPane). They flank the same
// vertical divider, so their heights must match; change the height here (not in
// one header) so the two can never drift apart. No bottom border: the headers
// blend into their panes for a freer, more "console"-like flow (Databricks
// style). Each header appends its own horizontal padding / typography via
// `cn(PANE_HEADER_CLASS, …)`.
export const PANE_HEADER_CLASS =
  "flex h-12 shrink-0 items-center justify-between";
