// `createScimIdentityHost` against a fake SCIM endpoint: one request per kind
// with a `co` filter, results mapped to principal refs, and `Me` as `current`.

import { expect, test } from "bun:test";
import { createScimIdentityHost, SCIM_BASE_PATH } from "./scim.js";

type Seen = { path: string; filter: string | null; count: string | null };

function fakeScim(routes: Record<string, unknown>, status = 200) {
  const seen: Seen[] = [];
  const fetch = (async (input: RequestInfo | URL) => {
    const url = new URL(String(input), "http://uc.test");
    const path = url.pathname.slice(SCIM_BASE_PATH.length);
    seen.push({
      path,
      filter: url.searchParams.get("filter"),
      count: url.searchParams.get("count"),
    });
    const body = routes[path];
    if (body === undefined) {
      return Response.json(
        { detail: "not found", status: "404" },
        { status: 404 },
      );
    }
    return Response.json(body, { status });
  }) as typeof globalThis.fetch;
  return { fetch, seen };
}

const list = (Resources: unknown[]) => ({
  totalResults: Resources.length,
  Resources,
});

test("search asks every kind and maps the resources", async () => {
  const { fetch, seen } = fakeScim({
    "/Users": list([
      { id: "u1", userName: "alice@example.com", displayName: "Alice" },
      { id: "u2", userName: "alfred@example.com" },
    ]),
    "/Groups": list([{ id: "g1", displayName: "all-analysts" }]),
    "/ServicePrincipals": list([{ id: "s1", displayName: "al-etl" }]),
  });
  const identity = createScimIdentityHost({ fetch });
  expect(await identity.search(' al"x ')).toEqual([
    { kind: "user", name: "alice@example.com", displayName: "Alice" },
    { kind: "user", name: "alfred@example.com" },
    { kind: "group", name: "all-analysts" },
    { kind: "service_account", name: "al-etl" },
  ]);
  expect(seen.map((s) => s.path).sort()).toEqual([
    "/Groups",
    "/ServicePrincipals",
    "/Users",
  ]);
  // Quotes are escaped inside the filter literal.
  expect(seen.find((s) => s.path === "/Users")?.filter).toBe(
    'userName co "al\\"x" or displayName co "al\\"x"',
  );
  expect(seen.find((s) => s.path === "/Groups")?.filter).toBe(
    'displayName co "al\\"x"',
  );
});

test("search honours kinds and limit, and skips an empty query", async () => {
  const { fetch, seen } = fakeScim({
    "/Users": list([
      { id: "u1", userName: "a@example.com" },
      { id: "u2", userName: "b@example.com" },
    ]),
  });
  const identity = createScimIdentityHost({ fetch });
  expect(await identity.search("  ")).toEqual([]);
  expect(seen).toHaveLength(0);
  const found = await identity.search("example", { kinds: ["user"], limit: 1 });
  expect(found).toEqual([{ kind: "user", name: "a@example.com" }]);
  expect(seen).toEqual([
    { path: "/Users", filter: expect.any(String), count: "1" },
  ]);
});

test("search fails with the server's detail", async () => {
  const { fetch } = fakeScim({ "/Users": { detail: "sign in first" } }, 401);
  const identity = createScimIdentityHost({ fetch });
  await expect(identity.search("a", { kinds: ["user"] })).rejects.toThrow(
    "sign in first",
  );
});

test("current is the caller, or null when signed out", async () => {
  const user = fakeScim({
    "/Me": { id: "u1", userName: "alice@example.com", displayName: "Alice" },
  });
  expect(
    await createScimIdentityHost({ fetch: user.fetch }).current?.(),
  ).toEqual({
    kind: "user",
    name: "alice@example.com",
    displayName: "Alice",
  });
  const account = fakeScim({
    "/Me": {
      schemas: ["urn:ietf:params:scim:schemas:core:2.0:ServicePrincipal"],
      id: "s1",
      displayName: "etl",
    },
  });
  expect(
    await createScimIdentityHost({ fetch: account.fetch }).current?.(),
  ).toEqual({ kind: "service_account", name: "etl" });
  const signedOut = fakeScim({ "/Me": { detail: "not signed in" } }, 401);
  expect(
    await createScimIdentityHost({ fetch: signedOut.fetch }).current?.(),
  ).toBeNull();
});
