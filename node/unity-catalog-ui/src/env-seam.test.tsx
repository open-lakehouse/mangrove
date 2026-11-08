// The environment scope resolves to: the provider's `scopeId`, else the host's
// environment key, else a constant. Tree expansion is persisted per scope, so
// one environment's expanded nodes never show in another.

import { afterEach, beforeEach, expect, test } from "bun:test";
import { registerHost } from "@open-lakehouse/host";
import { HostProvider } from "@open-lakehouse/host/react";
import { createFakeHost } from "@open-lakehouse/host/testing";
import type { ReactNode } from "react";
import { renderToString } from "react-dom/server";
import { ExpansionProvider, useExpansion } from "./ExpansionContext.js";
import { EnvironmentScopeProvider, useEnvironmentScopeId } from "./env-seam.js";

afterEach(() => registerHost(null));

/** Render `tree` and return what `read` saw inside it. */
function readIn<T>(read: () => T, wrap: (probe: ReactNode) => ReactNode): T {
  const seen: { value?: T } = {};
  function Probe() {
    seen.value = read();
    return null;
  }
  renderToString(wrap(<Probe />));
  if (!("value" in seen)) throw new Error("probe did not render");
  return seen.value as T;
}

const scopeIn = (wrap: (probe: ReactNode) => ReactNode) =>
  readIn(useEnvironmentScopeId, wrap);

test("without a provider or a host the scope is the shared default", () => {
  expect(scopeIn((probe) => probe)).toBe("default");
  expect(
    scopeIn((probe) => (
      <EnvironmentScopeProvider>{probe}</EnvironmentScopeProvider>
    )),
  ).toBe("default");
});

test("an explicit scopeId wins over the host", () => {
  const host = createFakeHost();
  expect(
    scopeIn((probe) => (
      <HostProvider host={host}>
        <EnvironmentScopeProvider scopeId="uc">
          {probe}
        </EnvironmentScopeProvider>
      </HostProvider>
    )),
  ).toBe("uc");
});

test("the provider defaults to the host's environment key", () => {
  const host = createFakeHost();
  const wrap = (probe: ReactNode) => (
    <HostProvider host={host}>
      <EnvironmentScopeProvider>{probe}</EnvironmentScopeProvider>
    </HostProvider>
  );
  const first = scopeIn(wrap);
  expect(first).toBe(host.environment.key);
  host.setEnvironment({ id: "other", name: "other" });
  expect(scopeIn(wrap)).toBe(host.environment.key);
  expect(scopeIn(wrap)).not.toBe(first);
});

test("without a provider the scope still follows a registered host", () => {
  const host = createFakeHost();
  registerHost(host);
  expect(scopeIn((probe) => probe)).toBe(host.environment.key);
});

// Tree expansion reads sessionStorage, which bun doesn't have: stand one in.
let storage: Map<string, string>;
const hadWindow = "window" in globalThis;
beforeEach(() => {
  storage = new Map();
  (globalThis as { window?: unknown }).window = {
    sessionStorage: {
      getItem: (k: string) => storage.get(k) ?? null,
      setItem: (k: string, v: string) => void storage.set(k, v),
    },
  };
});
afterEach(() => {
  if (!hadWindow) delete (globalThis as { window?: unknown }).window;
});

test("tree expansion is persisted per environment", () => {
  const host = createFakeHost();
  const isOpen = () => useExpansion().isOpen("catalog:main");
  const wrap = (probe: ReactNode) => (
    <HostProvider host={host}>
      <ExpansionProvider>{probe}</ExpansionProvider>
    </HostProvider>
  );
  storage.set(
    `catalog.expanded:${host.environment.key}`,
    JSON.stringify(["catalog:main"]),
  );
  expect(readIn(isOpen, wrap)).toBe(true);
  host.setEnvironment({ id: "other", name: "other" });
  expect(readIn(isOpen, wrap)).toBe(false);
});
