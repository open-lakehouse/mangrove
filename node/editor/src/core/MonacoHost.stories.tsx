// Stories for the editor CORE in isolation — no session shell, no host app.
// A fixture catalog provider is registered, a canned SQL model is created via
// the model registry, and `<MonacoHost>` displays it. This exercises the whole
// core path: monaco bootstrap, the pgsql worker (syntax highlighting + in-worker
// diagnostics squiggles), catalog-aware completion (type `main.` / `select `),
// and the ui-kit theme bridge.
//
// This is the acceptance harness that the SQL worker actually RUNS (not a
// main-thread fallback): if the worker fails to initialize, there are no
// diagnostics and completion returns only what the (empty-by-default) provider
// gives. Storybook is not yet wired in this repo; like the other packages'
// stories this is the reference harness and is excluded from tsc.

import { loader } from "@monaco-editor/react";
import { ThemeProvider } from "@open-lakehouse/ui-kit";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";
import { fixtureCatalogProvider } from "../fixtures";
import { registerCatalogProvider } from "./catalogProvider";
import { MonacoHost } from "./MonacoHost";
import { ensureModel } from "./models";

const SAMPLE_SQL = `-- Try editing: catalog-aware completion + live validation.
-- Type "main." to complete schemas, then tables, then columns.
select id, email, created_at
from main.default.users
where events > 10
order by created_at desc;
`;

// Register the fixture provider so completion has real names to offer.
registerCatalogProvider(fixtureCatalogProvider);

function CoreEditor() {
  const [ready, setReady] = useState(false);
  const path = "story://sample.sql";

  useEffect(() => {
    // Create the model once monaco is loaded, then point the host at it.
    loader.init().then((monaco) => {
      ensureModel(monaco, path, SAMPLE_SQL);
      setReady(true);
    });
  }, []);

  return (
    <div style={{ height: 420, border: "1px solid var(--border)" }}>
      <MonacoHost
        activeId={ready ? path : null}
        onRun={() => console.log("run (story no-op)")}
        emptyState="Loading…"
      />
    </div>
  );
}

const meta: Meta<typeof CoreEditor> = {
  title: "editor/MonacoHost (core)",
  component: CoreEditor,
  decorators: [
    (Story) => (
      <ThemeProvider>
        <Story />
      </ThemeProvider>
    ),
  ],
};
export default meta;

type Story = StoryObj<typeof CoreEditor>;

export const SqlWithCatalogCompletion: Story = {};
