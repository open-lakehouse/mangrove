// Fixtures for UIs built on this client: a small Unity Catalog served by a
// fetch-shaped handler, for `@open-lakehouse/host/testing`'s fake host
// (`createFakeHost({ services: { "unity-catalog": ucFixtureHandler() } })`), a
// story or a test. Read-only: list and detail GETs, 404 for anything else.
//
// The catalog: `main` (with schema `sales`: table `orders`, view
// `orders_summary`, volume `raw_files`) and an empty `scratch`.

import type {
  CatalogInfo,
  ColumnInfo,
  SchemaInfo,
  TableInfo,
  VolumeInfo,
} from "./uc-types.js";

const column = (
  name: string,
  type_name: ColumnInfo["type_name"],
  type_text: string,
  position: number,
  comment?: string,
): ColumnInfo => ({
  name,
  type_name,
  type_text,
  type_json: JSON.stringify({ name, type: type_text, nullable: true }),
  position,
  nullable: true,
  ...(comment ? { comment } : {}),
});

// Most UC timestamps are int64 millis, which the OpenAPI spec types as strings;
// schemas carry a number.
const created_ms = Date.UTC(2026, 0, 15);
const created_at = String(created_ms);

/** The fixture catalogs. */
export const fixtureCatalogs: CatalogInfo[] = [
  {
    name: "main",
    owner: "data-platform",
    comment: "Production data",
    storage_root: "s3://fixture-lakehouse/main",
    created_at,
  },
  { name: "scratch", owner: "analysts", created_at },
];

/** The fixture schemas. */
export const fixtureSchemas: SchemaInfo[] = [
  {
    name: "sales",
    catalog_name: "main",
    full_name: "main.sales",
    owner: "sales-eng",
    comment: "Orders and invoices",
    created_at: created_ms,
  },
];

/** The fixture tables (one managed table and one view). */
export const fixtureTables: TableInfo[] = [
  {
    name: "orders",
    catalog_name: "main",
    schema_name: "sales",
    full_name: "main.sales.orders",
    table_type: "MANAGED",
    data_source_format: "DELTA",
    storage_location: "s3://fixture-lakehouse/main/sales/orders",
    comment: "One row per order",
    owner: "sales-eng",
    created_at,
    columns: [
      column("order_id", "LONG", "bigint", 0, "Primary key"),
      column("customer_id", "LONG", "bigint", 1),
      column("amount", "DECIMAL", "decimal(10,2)", 2),
      column("ordered_at", "TIMESTAMP", "timestamp", 3),
    ],
  },
  {
    name: "orders_summary",
    catalog_name: "main",
    schema_name: "sales",
    full_name: "main.sales.orders_summary",
    table_type: "VIEW",
    data_source_format: "DELTA",
    view_definition:
      "SELECT customer_id, sum(amount) AS total FROM main.sales.orders GROUP BY customer_id",
    created_at,
    columns: [
      column("customer_id", "LONG", "bigint", 0),
      column("total", "DECIMAL", "decimal(20,2)", 1),
    ],
  },
];

/** The fixture volumes. */
export const fixtureVolumes: VolumeInfo[] = [
  {
    name: "raw_files",
    catalog_name: "main",
    schema_name: "sales",
    full_name: "main.sales.raw_files",
    volume_id: "00000000-0000-4000-8000-000000000001",
    volume_type: "EXTERNAL",
    storage_location: "s3://fixture-lakehouse/landing/sales",
    comment: "Landing zone for order exports",
    created_at,
  },
];

/** The path prefix the UC REST API is served under. */
const API_ROOT = "/api/2.1/unity-catalog";

const notFound = (what: string) =>
  Response.json(
    { error_code: "NOT_FOUND", message: `${what} does not exist` },
    { status: 404 },
  );

/** `items` filtered to those whose `field`s match the query params given. */
function where<T extends object>(
  items: T[],
  params: URLSearchParams,
  fields: (keyof T & string)[],
): T[] {
  return items.filter((item) =>
    fields.every((field) => {
      const wanted = params.get(field);
      return wanted === null || String(item[field]) === wanted;
    }),
  );
}

/** One list or detail route: list `items` under `key`, or one by `idField`. */
function collection<T extends object>(
  key: string,
  items: T[],
  idField: keyof T & string,
  filters: (keyof T & string)[],
) {
  return (id: string | undefined, params: URLSearchParams) => {
    if (id === undefined) {
      return Response.json({ [key]: where(items, params, filters) });
    }
    const hit = items.find((item) => String(item[idField]) === id);
    return hit ? Response.json(hit) : notFound(`${key} '${id}'`);
  };
}

const routes: Record<
  string,
  (id: string | undefined, params: URLSearchParams) => Response
> = {
  catalogs: collection("catalogs", fixtureCatalogs, "name", []),
  schemas: collection("schemas", fixtureSchemas, "full_name", ["catalog_name"]),
  tables: collection("tables", fixtureTables, "full_name", [
    "catalog_name",
    "schema_name",
  ]),
  volumes: collection("volumes", fixtureVolumes, "full_name", [
    "catalog_name",
    "schema_name",
  ]),
  functions: () => Response.json({ functions: [] }),
  models: () => Response.json({ registered_models: [] }),
  credentials: () => Response.json({ credentials: [] }),
  "external-locations": () => Response.json({ external_locations: [] }),
};

/**
 * A fetch-shaped handler serving the fixture catalog's read API under
 * `/api/2.1/unity-catalog`. Writes and unknown paths answer 404.
 */
export function ucFixtureHandler(): (request: Request) => Response {
  return (request) => {
    const url = new URL(request.url);
    const path = url.pathname.startsWith(API_ROOT)
      ? url.pathname.slice(API_ROOT.length)
      : url.pathname;
    const [, resource, ...rest] = path.split("/");
    const route = resource ? routes[resource] : undefined;
    if (request.method !== "GET" || !route) return notFound(path);
    const id = rest.length ? decodeURIComponent(rest.join("/")) : undefined;
    return route(id, url.searchParams);
  };
}
