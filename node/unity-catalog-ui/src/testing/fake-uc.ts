// A fake Unity Catalog for dialog tests. Reads are served from the client
// package's fixture catalog (plus any overrides), writes are recorded and
// answered with a canned response, so a test can assert the exact request a
// form sent.

import { ucFixtureHandler } from "@open-lakehouse/unity-catalog-client/testing";

export interface RecordedRequest {
  method: string;
  /** Path below the UC API root, e.g. `/schemas/main.sales`. */
  path: string;
  query: Record<string, string>;
  body: unknown;
}

type Responder = (request: RecordedRequest) => Response | undefined;

const API_ROOT = "/api/2.1/unity-catalog";

export const UC_BASE_URL = `http://uc.test${API_ROOT}`;

export function createFakeUc() {
  const requests: RecordedRequest[] = [];
  const responders: Responder[] = [];
  const fixtures = ucFixtureHandler();

  async function fetch(input: RequestInfo | URL, init?: RequestInit) {
    const request = new Request(input, init);
    const url = new URL(request.url);
    const text = request.method === "GET" ? "" : await request.text();
    const recorded: RecordedRequest = {
      method: request.method,
      path: url.pathname.slice(API_ROOT.length),
      query: Object.fromEntries(url.searchParams),
      body: text ? JSON.parse(text) : undefined,
    };
    requests.push(recorded);
    for (const respond of responders) {
      const response = respond(recorded);
      if (response) return response;
    }
    if (request.method === "GET") return fixtures(request);
    // Default write answer: echo the body, with a full name like the server.
    const body = (recorded.body ?? {}) as Record<string, unknown>;
    const parts = [body.catalog_name, body.schema_name, body.name].filter(
      (p): p is string => typeof p === "string",
    );
    return Response.json({ ...body, full_name: parts.join(".") });
  }

  return {
    fetch: fetch as typeof globalThis.fetch,
    /** Every request made so far. */
    requests,
    /** Requests other than GETs. */
    writes: () => requests.filter((r) => r.method !== "GET"),
    /** Answer matching requests first (return undefined to pass). */
    respond(responder: Responder) {
      responders.unshift(responder);
    },
  };
}

export function ucError(status: number, error_code: string, message: string) {
  return Response.json({ error_code, message }, { status });
}
