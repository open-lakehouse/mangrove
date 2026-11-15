// `useIdentityHost` reads, in order: `<IdentityProvider>`, the host's
// `identity` capability, else nothing.

import { expect, test } from "bun:test";
import type { LakehouseHost } from "@open-lakehouse/host";
import { HostProvider } from "@open-lakehouse/host/react";
import { createFakeHost } from "@open-lakehouse/host/testing";
import type { ReactNode } from "react";
import { renderToString } from "react-dom/server";
import type { IdentityHost } from "../identity.js";
import { IdentityProvider, useIdentityHost } from "./identity.js";

const directory = (name: string): IdentityHost => ({
  search: async () => [{ kind: "user", name }],
});

function identityIn(wrap: (probe: ReactNode) => ReactNode) {
  let seen: IdentityHost | undefined | null = null;
  function Probe() {
    seen = useIdentityHost();
    return null;
  }
  renderToString(wrap(<Probe />));
  return seen as IdentityHost | undefined;
}

const hostWithIdentity = (identity: IdentityHost): LakehouseHost =>
  Object.assign(createFakeHost({ services: {} }), { identity });

test("without a provider or a host identity there is no directory", () => {
  expect(identityIn((probe) => probe)).toBeUndefined();
  expect(
    identityIn((probe) => (
      <HostProvider host={createFakeHost({ services: {} })}>
        {probe}
      </HostProvider>
    )),
  ).toBeUndefined();
});

test("the host's identity capability is used", () => {
  const fromHost = directory("host");
  expect(
    identityIn((probe) => (
      <HostProvider host={hostWithIdentity(fromHost)}>{probe}</HostProvider>
    )),
  ).toBe(fromHost);
});

test("an IdentityProvider wins over the host", () => {
  const provided = directory("provided");
  expect(
    identityIn((probe) => (
      <HostProvider host={hostWithIdentity(directory("host"))}>
        <IdentityProvider identity={provided}>{probe}</IdentityProvider>
      </HostProvider>
    )),
  ).toBe(provided);
});
