// Databricks-style breadcrumb trail shown above a detail/page title. Each crumb
// is a link back up the hierarchy; the current item is rendered separately as
// the large title (so the trail ends with a trailing chevron, not the current
// name). Crumbs navigate either via an in-app route (`to`) or an imperative
// callback (`onClick`, e.g. the URL-addressable catalog selection).
import { Link } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { Fragment } from "react";

export interface Crumb {
  label: string;
  /** Imperative navigation (used by the catalog selection, which lives in the URL). */
  onClick?: () => void;
  /** In-app route target (used to jump between top-level pages). */
  to?: string;
  search?: Record<string, unknown>;
}

const CRUMB_CLASS =
  "min-w-0 truncate rounded-sm text-primary transition-colors hover:text-primary/80 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

const CHEVRON_CLASS = "h-3.5 w-3.5 shrink-0 text-muted-foreground/60";

export function Breadcrumbs({ items }: { items: Crumb[] }) {
  if (items.length === 0) return null;
  return (
    <nav
      aria-label="Breadcrumb"
      className="flex min-w-0 items-center gap-1 text-xs"
    >
      {items.map((item, i) => (
        <Fragment key={`${item.label}-${i}`}>
          {i > 0 && <ChevronRight className={CHEVRON_CLASS} aria-hidden />}
          {item.to ? (
            <Link to={item.to} search={item.search} className={CRUMB_CLASS}>
              {item.label}
            </Link>
          ) : (
            <button
              type="button"
              onClick={item.onClick}
              className={CRUMB_CLASS}
            >
              {item.label}
            </button>
          )}
        </Fragment>
      ))}
      {/* Trailing chevron connects the trail to the current item's title below. */}
      <ChevronRight className={CHEVRON_CLASS} aria-hidden />
    </nav>
  );
}
