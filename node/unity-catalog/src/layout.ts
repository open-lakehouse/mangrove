// Shared chrome for the sidebar tree header (CatalogTree). The detail pane leads
// with a breadcrumb + title stack instead (Databricks style), so it no longer
// shares this exact row — but keep this the single source for the sidebar header
// height. No bottom border: headers blend into their panes for a freer,
// "console"-like flow. Each consumer appends its own padding / typography via
// `cn(PANE_HEADER_CLASS, …)`.
export const PANE_HEADER_CLASS =
  "flex h-12 shrink-0 items-center justify-between";
