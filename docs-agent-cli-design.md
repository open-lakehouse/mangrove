# Agent-facing `uc` CLI — design

Status: **accepted**, 2026-07-24. First slice (the output layer) implemented; the
question-verbs and MCP server are staged follow-ups.

## Why

We want the `uc` CLI to be a first-class tool for AI agents — both interactive
agent sessions and autonomous agents a data practitioner points at a Unity
Catalog. The recurring jobs an agent takes on against a catalog are things like:
fill in missing table/column descriptions, discover "what data do we have?",
learn a table's schema before writing SQL, and audit metadata hygiene. Each of
those is a small investigation the CLI should answer directly.

The sibling `headwaters` repo (`hw`) already established the pattern
(`headwaters/docs/adr/0014-agent-facing-cli-question-verbs.md`); this ports the
philosophy to `uc`. It is also grounded in Anthropic's "Writing tools for
agents" (high-signal, token-efficient output; actionable errors; consolidate
workflows rather than wrap endpoints; a verbosity escape hatch) and the MCP tool
spec (structured output, stable error conventions).

## Decisions

1. **Three render modes behind one seam.** `--output` gains `agent` alongside
   `auto|json|table|plain`. A `Render` trait produces the agent view; `json`
   stays the faithful, byte-stable wire shape (the scripting contract). See
   `crates/cli/src/render.rs`.

2. **A versioned, pruned agent envelope.** Agent output is
   `{"_v": 1, "kind", "count"?, "data"}`. The `data` drops low-signal fields
   (UUIDs, timestamps, empty maps) and surfaces the answer, with a `_next` array
   of runnable follow-up `uc …` commands. `--raw` bypasses pruning and emits the
   full wire fields when an agent needs everything.

3. **Structured errors + stable exit codes.** In `json`/`agent` modes an error
   is `{"error", "kind"}` on stderr; the process exits `0` ok / `2` usage /
   `3` not-found / `4` auth / `1` other. `kind`/`exit_code` delegate to the
   client's typed predicates (`is_not_found`, `is_unauthenticated`,
   `is_permission_denied`, `is_already_exists`). Agents branch on these without
   parsing prose.

4. **A static capabilities primer.** `uc schema` emits (no server call) the
   identifier grammar, entity kinds, output modes, the envelope shape, the error
   contract, and a commands map (question + example per verb). Agents prime once
   instead of re-deriving the surface from `--help`.

5. **Quiet + token auth.** `--quiet` suppresses informational status chatter
   (errors are never suppressed). A bearer token from `--token` / `UC_TOKEN` /
   `DATABRICKS_TOKEN` lets agents hit non-local catalogs.

## Validation lenses (not commands)

The verb set is judged by whether these workflows are answerable by composing a
few verbs — they are how we test sufficiency, not features in themselves:

1. Fill missing table & column descriptions.
2. Catalog discovery ("what data do we have?").
3. Schema/DDL awareness for query authoring.
4. Governance / metadata-hygiene audits.
5. Data profiling before analysis.

## Staged follow-ups

- **Question-verbs** (next PR): `describe`, `columns`, `list-tables --summaries`
  (using the lighter `list_table_summaries`), `find`, `set-comment`
  (table/schema/catalog now). Each returns the interpreted answer, not a raw
  endpoint dump.
- **`preview` / `sample`** (deferred): reads sample rows by vending temporary
  credentials (`generate_temporary_table_credentials` /
  `delta_v1().get_table_credentials()`) and driving the Delta reader. Needs the
  Delta read stack wired into the native CLI. Designed, not built.
- **Column-level `set-comment --column`** (deferred): there is **no column-comment
  API** today (client + proto/server gap). A follow-up issue tracks adding it;
  the verb is designed against that.
- **`uc mcp` stdio server** (deferred, designed-for): the `Render` trait + the
  question-verbs are exactly the seam an MCP server would wrap — one tool per
  verb, the agent envelope as the tool result, `uc schema` as discovery.

## Explicitly out of scope (for now)

- `agents` / `agent_skills` verbs — the API is too unstable to surface yet.
- Governance-resource and volume-lifecycle verbs.

This document was AI-assisted by Isaac.
