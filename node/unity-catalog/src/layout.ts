// Shared chrome for the explorer's two column headers — the sidebar tree header
// (CatalogTree) and the detail-pane header (DetailPane). They sit either side of
// the same divider, so their heights must match. Both consume
// PANE_HEADER_CLASS; change the height here (not in one header) so the two can
// never drift apart. Each header appends its own horizontal padding / typography
// via `cn(PANE_HEADER_CLASS, …)`.
export const PANE_HEADER_CLASS =
  "flex h-12 shrink-0 items-center justify-between border-b";
