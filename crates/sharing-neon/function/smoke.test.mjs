// Smoke test for the built function bundle (`dist/`): load the addon through
// the real fetch handler and drive a provider → recipient round trip on the
// in-memory store. Run with `just sharing-neon test` (host) or
// `just sharing-neon test-arm64` (the shipped linux-arm64 bundle).

import assert from "node:assert/strict";
import { test } from "node:test";

process.env.SHARING_SERVER_CONFIG_YAML ??= [
  "public_url: https://sharing.example.com",
  "provider_tokens: [smoke-provider]",
].join("\n");

const { default: handler } = await import("./dist/index.mjs");

async function call(method, path, { token, body } = {}) {
  const headers = { "content-type": "application/json" };
  if (token) headers.authorization = `Bearer ${token}`;
  const response = await handler.fetch(
    new Request(`https://sharing.example.com${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    }),
  );
  const text = await response.text();
  const isJson = response.headers.get("content-type")?.includes("json");
  return { status: response.status, json: isJson ? JSON.parse(text) : undefined, text };
}

test("health", async () => {
  const response = await call("GET", "/health");
  assert.equal(response.status, 200);
  assert.equal(response.text, "ok");
});

test("provider creates a share, recipient activates and lists it", async () => {
  const admin = (method, path, body) =>
    call(method, `/api/2.1/unity-catalog${path}`, { token: "smoke-provider", body });

  assert.equal((await call("GET", "/api/2.1/unity-catalog/shares")).status, 401);
  assert.equal((await admin("POST", "/shares", { name: "smoke" })).status, 200);
  const recipient = await admin("POST", "/recipients", {
    name: "acme",
    authentication_type: "TOKEN",
  });
  assert.equal(recipient.status, 200, recipient.text);
  const grant = await admin("PATCH", "/shares/smoke/permissions", {
    changes: [{ principal: "acme", add: ["SELECT"] }],
  });
  assert.equal(grant.status, 200, grant.text);

  const link = new URL(recipient.json.tokens[0].activation_url);
  const profile = await call("GET", link.pathname);
  assert.equal(profile.status, 200, profile.text);
  assert.equal(profile.json.endpoint, "https://sharing.example.com/api/v1/delta-sharing");

  const shares = await call("GET", "/api/v1/delta-sharing/shares", {
    token: profile.json.bearerToken,
  });
  assert.equal(shares.status, 200, shares.text);
  assert.deepEqual(
    shares.json.items.map((s) => s.name),
    ["smoke"],
  );
});
