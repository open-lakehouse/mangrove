import { LogQueryServiceProvider } from "@open-lakehouse/delta-log";
import { registerStubLogPreview } from "@open-lakehouse/delta-log/testing";
import { HostProvider } from "@open-lakehouse/host/react";
import { createFakeHost } from "@open-lakehouse/host/testing";
import { QueryServiceProvider } from "@open-lakehouse/query-runner";
import {
  ThemeProvider,
  Toaster,
  TooltipProvider,
} from "@open-lakehouse/ui-kit";
import {
  UNITY_CATALOG_MODULE_ID,
  UnityCatalogProvider,
} from "@open-lakehouse/unity-catalog-client/react";
import { ucFixtureHandler } from "@open-lakehouse/unity-catalog-client/testing";
import { EnvironmentScopeProvider } from "@open-lakehouse/unity-catalog-ui";
import { FilesServiceProvider } from "@open-lakehouse/volume-files";
import { registerStubFiles } from "@open-lakehouse/volume-files/testing";
import type { Preview } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type ReactNode, useState } from "react";
import "../src/globals.css";

// Every story renders under a fake host (a storybook-kind LakehouseHost), the
// same seam the desktop shell fills. It binds "unity-catalog" to the fixture
// catalog, so UnityCatalogProvider and the environment scope pick it up with no
// story-level wiring. A story that needs other services or another environment
// passes its own `<HostProvider host={createFakeHost(…)}>`.
const host = createFakeHost({
  services: { [UNITY_CATALOG_MODULE_ID]: ucFixtureHandler() },
});

// The Delta-log tab and the volume Files tab are gated on a registered runner;
// the dev stubs serve canned data, as in the app's default build.
registerStubLogPreview();
registerStubFiles();

/** A query cache per story, so one story's fetched data never shows in another. */
function StoryQueryClient({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const preview: Preview = {
  decorators: [
    (Story) => (
      <HostProvider host={host}>
        <StoryQueryClient>
          <UnityCatalogProvider>
            <QueryServiceProvider>
              <LogQueryServiceProvider>
                <FilesServiceProvider>
                  <EnvironmentScopeProvider>
                    <ThemeProvider>
                      <TooltipProvider delayDuration={300}>
                        <Story />
                        <Toaster position="bottom-right" />
                      </TooltipProvider>
                    </ThemeProvider>
                  </EnvironmentScopeProvider>
                </FilesServiceProvider>
              </LogQueryServiceProvider>
            </QueryServiceProvider>
          </UnityCatalogProvider>
        </StoryQueryClient>
      </HostProvider>
    ),
  ],
};

export default preview;
