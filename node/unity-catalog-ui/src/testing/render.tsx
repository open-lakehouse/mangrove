// Render a component the way the Catalog Explorer mounts it: a fresh query
// client, a UC client over a fake transport, the `/catalog` route and tree
// expansion.

import {
  createUnityCatalogClient,
  UnityCatalogProvider,
} from "@open-lakehouse/unity-catalog-client/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type RenderResult, render, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { ExpansionProvider } from "../ExpansionContext.js";
import { createFakeUc, UC_BASE_URL } from "./fake-uc.js";
import { CatalogRouterHarness } from "./story-router.js";

export type FakeUc = ReturnType<typeof createFakeUc>;

export function renderWithUc(
  ui: ReactNode,
  opts: { initialSel?: string } = {},
): RenderResult & { uc: FakeUc; queryClient: QueryClient } {
  const uc = createFakeUc();
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const client = createUnityCatalogClient({
    baseUrl: UC_BASE_URL,
    fetch: uc.fetch,
  });
  const result = render(
    <QueryClientProvider client={queryClient}>
      <UnityCatalogProvider client={client}>
        <CatalogRouterHarness initialSel={opts.initialSel}>
          <ExpansionProvider>{ui}</ExpansionProvider>
        </CatalogRouterHarness>
      </UnityCatalogProvider>
    </QueryClientProvider>,
  );
  return { ...result, uc, queryClient };
}

type Screen = ReturnType<typeof within>;

/**
 * Testing Library's `screen`, bound lazily. Its own `screen` binds to the
 * `document` that exists when the module loads, and Bun loads that CommonJS
 * module before `testing/dom.ts` registers happy-dom.
 */
export const screen = new Proxy({} as Screen, {
  get: (_, key) => Reflect.get(within(document.body), key),
});
