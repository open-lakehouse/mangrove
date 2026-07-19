// A hardcoded `CatalogProvider` for stories, tests, and dev — shipped behind the
// `./fixtures` subexport so it never lands in a production consumer's bundle
// (mirrors log-query's `./testing` and query-wasm's `./stub` conventions).
//
// Register it with `registerCatalogProvider(fixtureCatalogProvider)` to get
// catalog-aware completion without a live backend.

import type { CatalogColumn, CatalogProvider } from "./core/catalogProvider";

interface FixtureTable {
  columns: CatalogColumn[];
}
// catalog → schema → table → columns
const FIXTURE: Record<string, Record<string, Record<string, FixtureTable>>> = {
  main: {
    default: {
      users: {
        columns: [
          { name: "id", type: "bigint" },
          { name: "email", type: "string" },
          { name: "created_at", type: "timestamp" },
          { name: "events", type: "bigint" },
        ],
      },
      events: {
        columns: [
          { name: "id", type: "bigint" },
          { name: "user_id", type: "bigint" },
          { name: "name", type: "string" },
          { name: "ts", type: "timestamp" },
        ],
      },
    },
    analytics: {
      daily_active: {
        columns: [
          { name: "date", type: "date" },
          { name: "count", type: "bigint" },
        ],
      },
    },
  },
  samples: {
    nyctaxi: {
      trips: {
        columns: [
          { name: "pickup_zip", type: "int" },
          { name: "dropoff_zip", type: "int" },
          { name: "fare_amount", type: "double" },
          { name: "trip_distance", type: "double" },
        ],
      },
    },
  },
};

export const fixtureCatalogProvider: CatalogProvider = {
  async catalogs() {
    return Object.keys(FIXTURE);
  },
  async schemas(catalog) {
    return Object.keys(FIXTURE[catalog] ?? {});
  },
  async tables(catalog, schema) {
    return Object.keys(FIXTURE[catalog]?.[schema] ?? {});
  },
  async columns(fullTableName) {
    const [c, s, t] = fullTableName.split(".");
    return FIXTURE[c]?.[s]?.[t]?.columns ?? [];
  },
};
