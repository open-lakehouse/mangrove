# `uc mcp` — MCP server for Unity Catalog — design

Status: **proposed**, 2026-10-03; §8 (grants as an interactive surface) is
**implemented**, 2026-11-15, in `crates/cli/src/mcp` and `node/uc-a2ui`, and
the Streamable HTTP transport in §1 (`--http`) on 2026-11-16. The rest is
design. Builds on
[`docs-agent-cli-design.md`](docs-agent-cli-design.md), which staged a `uc mcp`
stdio server as a designed-for follow-up.

## Why

Agents working against a Unity Catalog want one local, governed entry point:
browse and annotate metadata, and call the functions a team has registered in
the catalog as tools. The agent-CLI work already gave us the pieces an MCP
server wraps — the `Render` trait and agent envelope
(`crates/cli/src/render.rs`), the structured error contract, and the `uc schema`
primer. What is new is **UC functions as tools**: SQL- and Python-bodied
functions whose code comes from a remote catalog and runs on the user's machine
under the user's identity. Most of this document is about doing that safely.

We want this in the existing `uc` binary (Rust), not as a separate Python
process: one install, the same auth, and the same output contract for humans,
scripts, and agents.

## Prior art

| | Databricks managed MCP | unitycatalog-ai (OSS) | Polaris (`polaris-tools/mcp-server`) | Gravitino MCP | DataHub MCP |
|---|---|---|---|---|---|
| Tool shape | Dynamic: one tool per UC function in a schema; plus DBSQL, Vector/AI Search, Genie servers | Toolkits (LangChain, OpenAI, Anthropic, …) built from UC functions | 7 coarse `*-request` tools with an `operation` argument | ~25 fine-grained tools (catalogs, schemas, tables, models, filesets, tags, policies, statistics, jobs) | ~10+ discovery/lineage tools, plus mutation tools |
| Mutations | Only what a function does | n/a | Full CRUD incl. principals/roles, no gates | Some (tag association, `run_job`) | Off by default (`TOOLS_IS_MUTATION_ENABLED`); `readOnlyHint`/`destructiveHint` |
| Function execution | Serverless / SQL warehouse, UC `EXECUTE` checked | Python only. `local` = in-process `exec`; `sandbox` = subprocess + `setrlimit` + module blocklist | — | — | — |
| Isolation | Remote compute | Process + rlimits; **no FS/network isolation** | — | — | — |
| Auth | OAuth / PAT, on behalf of the user | UC client creds | Static token or OAuth client credentials | `Authorization` header passthrough | OAuth + DCR, or PAT |
| Notable | Scoped per `{catalog}/{schema}` endpoint | Wraps the body in a `def` from the params | Returns HTTP transcript + structured result | Writes an audit log of every tool call | Default-view scoping for service accounts |

What we adopt: dynamic function tools scoped per schema (Databricks);
read-only by default, writes opt-in, and MCP tool hints (DataHub); an audit log
(Gravitino); fine-grained verbs instead of a CRUD `operation` switch (the
opposite of Polaris — the agent-CLI design already argued for question-verbs
over endpoint wrappers). From unitycatalog-ai we take the function-body
convention so a function behaves the same under both, but not its isolation
model.

> Verify before accepting: the exact Databricks tool-naming scheme and how it
> maps parameter types, and the current list of unitycatalog-ai `EXECUTOR_*`
> settings. Both came from secondary sources.

## Decisions

### 1. `uc mcp serve`, in the existing CLI, behind an `mcp` feature

A new subcommand built on `rmcp` (the official Rust MCP SDK), compiled in
behind a cargo `mcp` feature so the base CLI stays lean. v1 transport is
**stdio**: the MCP client spawns `uc mcp serve`, which inherits the user's
credentials through the existing `GlobalOpts::client()` path (`--token` /
`UC_TOKEN` / `DATABRICKS_TOKEN`).

A client config entry stays a one-liner:

```json
{ "command": "uc", "args": ["mcp", "serve", "--functions", "main.tools"] }
```

**Streamable HTTP (implemented)** lets several clients share one server, for
example an agent and the MCP Apps host rendering its forms:

```sh
uc mcp serve --http 0.0.0.0:8090 [--path /mcp] [--allowed-host uc-mcp]...
```

- rmcp's `StreamableHttpService` with a local session manager and a 15s SSE
  keep-alive, at `--path` (default `/mcp`), plus `GET /health` (200 `ok`).
  It serves legacy sessions and the stateless 2026-07-28 protocol, and shuts
  down gracefully on ctrl-c.
- **No inbound auth.** Outbound credentials are fixed at startup
  (`--server` / `--token`), so every client acts as that identity. Run it
  behind a gateway that authenticates callers, for one user's environment,
  never as a shared deployment.
- **A `Host` allowlist** guards against DNS rebinding: loopback only by
  default; `--allowed-host` (repeatable, `host` or `host:port`) replaces it,
  for example with the service name a container network uses. `/health` is
  not checked.
- **Gating per connection.** Whether a client sees the app-only tools (§8)
  comes from its own capabilities: the request's `_meta` in the stateless
  protocol, else the session's `initialize`. An agent and an MCP Apps host on
  one server each see their own tool list. The same gate decides whether a
  tool result carries the A2UI resource (§8).

OAuth (the MCP authorization spec) and per-request bearer forwarding stay
staged: they need a real resource-server story, and a shared server changes
the threat model in §4.

### 2. Three tool families, three trust levels

- **Catalog tools** — one tool per agent-CLI question-verb (`describe`,
  `columns`, `find`, `list_catalogs`/`list_schemas`/`list_tables`, …). The tool
  result is the agent envelope as `structuredContent`, with a text rendering
  for clients that ignore structured output. All carry `readOnlyHint`. The
  `uc schema` primer becomes the server `instructions`. Verbs and tools share
  one implementation, so there is a single place for "what's useful to an
  agent".
- **Mutation tools** (`set_comment`, later tags) — registered only with
  `--allow-writes`, and marked `readOnlyHint: false`. Tools that overwrite
  existing values (such as a comment) also get `destructiveHint: true`, so
  clients can ask for confirmation.
- **Function tools** — generated from UC functions, and **only** for functions
  matched by an explicit allowlist: `--functions <catalog>.<schema>` or
  `<catalog>.<schema>.<function>`, repeatable. No allowlist means no function
  tools.

Tables can optionally be exposed as MCP resources (`uc://<catalog>/<schema>/<table>`,
returning the `describe` output) for clients that attach context rather than
call tools.

### 3. Mapping a UC function to an MCP tool

- **Name:** `<catalog>__<schema>__<function>`, restricted to `[A-Za-z0-9_-]`.
  Names over the client length limit are truncated and suffixed with a short
  hash of the full name, so they stay unique and stable.
- **Description:** the function `comment`, followed by the SQL-style signature
  and return type. A function with no comment still gets a usable description,
  but `uc` warns about it, since a missing comment is a metadata-hygiene
  problem the agent-CLI lenses already target.
- **`inputSchema`:** derived from `input_params[].type_json`, using one
  UC-type → JSON-Schema table that lives next to the type parser:

  | UC type | JSON Schema |
  |---|---|
  | `TINYINT`/`SMALLINT`/`INT`/`BIGINT` | `integer` (with range bounds) |
  | `FLOAT`/`DOUBLE` | `number` |
  | `DECIMAL(p,s)` | `string` with a decimal `pattern` (so precision isn't lost) |
  | `STRING`/`CHAR`/`VARCHAR` | `string` |
  | `BOOLEAN` | `boolean` |
  | `DATE` / `TIMESTAMP` | `string`, `format: date` / `date-time` |
  | `BINARY` | `string`, `contentEncoding: base64` |
  | `ARRAY<T>` / `MAP<STRING,T>` / `STRUCT<…>` | `array` / `object` with `additionalProperties` / `object` with `properties` |

  Parameters with a `parameter_default` are optional. Parameter `comment`s
  become property descriptions.
- **Result:** scalar returns go into `structuredContent` as
  `{"result": <value>}`. Table-valued functions return
  `{"columns": […], "rows": […], "truncated": bool}`, capped by a configurable
  row/byte limit so the result fits in the model's context. Failures set
  `isError` and reuse the CLI's `{error, kind}` contract, adding kinds such as
  `timeout`, `sandbox_denied`, `unsupported`.
- **Refresh:** the function list is loaded at startup and refreshed on a TTL
  or explicitly. Changes send `notifications/tools/list_changed`.

### 4. Trust model: allowlist, pin, audit

The threat is plain: anyone who can `CREATE FUNCTION` in an allowlisted schema
can author code that `uc mcp` will run locally, as the user, with the user's
catalog token in reach. Tool arguments come from an LLM and may carry prompt
injection. Isolation (§5) limits what code can do. The trust layer decides
which code runs at all.

- **Scope** — the schema/function allowlist from §2, plus an optional owner
  allowlist (`--trusted-owner`), so only functions owned by named
  principals/groups are exposed.
- **Pinning** — a local trust file records, per function, a digest of
  (`routine_definition`, parameter signature, declared dependencies,
  language). A function whose digest is new or has changed is listed as
  disabled. The user runs `uc mcp trust <function>`, which shows the body and
  dependencies, and only then is it callable. This is trust-on-first-use with
  an explicit prompt, the same idea as SSH host keys. `--trust-on-first-use`
  auto-accepts for local development.
- **Audit** — every tool call is appended to a JSONL log: tool, a digest of the
  arguments, the function digest, sandbox backend, duration, outcome. The
  arguments themselves are only logged with `--audit-args`, since they may
  contain sensitive values.
- **No token in the sandbox** — the catalog token never enters the function's
  environment. The runner gets a scrubbed environment (§5).

### 5. Python functions: two phases, uv for environments, an OS sandbox for execution

The obvious design — write the body to a PEP 723 script and `uv run --script`
it — gets environments right and isolation wrong. uv isolates *dependencies*,
not the *process*. The function would run with full filesystem and network
access, and installing dependencies needs the network while running user code
must not have it. So we split the two:

**Phase A — provision (network allowed, no function code runs).**
Resolve the function's declared dependencies into a **content-addressed
virtualenv**, keyed by `hash(python version, normalized dependency list)`,
under the `uc` cache dir. We drive `uv` as a subprocess (`uv venv --python
<ver>` then `uv pip install`). We do not link uv's crates, which are not a
stable library API. Functions with identical deps share an environment. Once
built, an environment is marked read-only and only ever re-created, never
mutated. `--require-pinned-deps` rejects unpinned requirements, and
`--offline` provisions from uv's cache only. Installing a package can run its
build code, so provisioning prefers wheels (`--only-binary :all:` by default,
opt-out per config). Phase A is where supply-chain risk lives, and it is
covered by the §4 pin: dependencies are part of the trusted digest.

**Phase B — execute (no network, no writes outside a scratch dir).**
`uc` generates a small runner. It defines the function from the parameter list
and body (the same wrapping convention as unitycatalog-ai), reads JSON
arguments from stdin, converts them using the §3 type table, calls the
function, and writes a JSON result to stdout. The runner is launched with the
environment's interpreter, inside a sandbox backend, with:

- a wall-clock timeout (process group killed on expiry);
- rlimits for CPU time, address space, open files, and process count;
- a cap on stdout/stderr bytes;
- a scrubbed environment (no tokens, no `HOME`-derived credentials, `TMPDIR`
  pointing at a per-call scratch dir that is deleted afterwards).

**Sandbox backends** sit behind one trait in a new crate (working name
`crates/function-runtime`), so the server can later reuse it for server-side
execution:

| Backend | Mechanism | Strength | Deps | Platforms | Role |
|---|---|---|---|---|---|
| `os` | macOS: Seatbelt profile via `sandbox-exec`. Linux: Landlock (FS) + seccomp (no `socket` for non-`AF_UNIX`), applied in a re-exec'd `uc` helper before `exec`; bubblewrap used when present for namespaces | Strong | Full (native wheels) | macOS, Linux | **Default** |
| `container` | `docker`/`podman run --network=none --read-only --tmpfs /tmp`, env mounted read-only; gVisor `runsc` when available | Strong to very strong | Full | Anywhere with a container runtime | Windows; shared hosts; high-risk catalogs |
| `wasm` | CPython-WASI (or Pyodide) under wasmtime, with fuel, memory limits, and WASI preopens only | Very strong, portable | Pure-Python, plus Pyodide's curated set | Anywhere | Staged |
| `none` | Plain subprocess | None | Full | Anywhere | Requires `--sandbox none`; prints a warning on every call; equivalent to unitycatalog-ai `local` |

The `os` backend follows two implementations that already ship: OpenAI Codex
CLI (Seatbelt on macOS, Landlock + seccomp on Linux) and Anthropic's
`sandbox-runtime` (Seatbelt / bubblewrap). The policy is the same on both
platforms: read-only access to the interpreter, stdlib, and the function's
environment; read-write only to the scratch dir; no network. If the platform
can't enforce that policy (an old kernel without Landlock, or Windows), `uc`
refuses to run the function and names the backends that would work. It never
falls back to `none` silently.

**Network egress** is off for functions in v1. Some legitimate functions call
an HTTP API. The staged answer is the `sandbox-runtime` pattern: the sandbox
can only reach a local proxy, and the proxy enforces a per-function host
allow-list stored in the trust file.

**Rejected as isolation boundaries:** module blocklists (unitycatalog-ai
`sandbox` mode — `importlib`, `ctypes`, and C extensions get around them),
RestrictedPython (an AST filter, not a boundary), and embedding CPython via
PyO3 (shared process, shared memory, shared token). Any of them can be added as
defense in depth. None of them is the boundary.

### 6. SQL functions: in-process DataFusion, scoped by `sql_data_access`

- **`NO_SQL` / `CONTAINS_SQL` scalar functions** are evaluated as
  `SELECT <body>` with bound parameters in an in-process DataFusion
  `SessionContext` that has no catalog or table providers registered. That is
  pure computation with no IO, so it needs no sandbox beyond timeouts and
  memory limits. UC function bodies are Spark SQL. Where DataFusion's dialect
  differs, planning fails and the tool returns an `unsupported` error. The
  function stays listed, flagged as such, so the gap is visible.
- **`READS_SQL_DATA` and table functions** are staged. They need the
  `crates/datafusion` UC catalog provider and temporary-credential vending in
  the native CLI — the same dependency the agent-CLI `preview` verb is waiting
  on. They will be built together.
- **Other `EXTERNAL` languages** (Java, Scala, …) are listed by `uc mcp
  functions` as unsupported and are not registered as tools.

### 7. Configuration

Flags for the common case, plus an optional `uc-mcp.toml` for allowlists,
trusted owners, sandbox backend and limits, provisioning policy, trust-file and
audit-log paths. Flags override the file. `uc mcp functions` (a plain CLI verb,
not a tool) shows what would be exposed, with each function's trust state,
language, and support status, so users can check the setup without an MCP
client.

### 8. Interactive surfaces: A2UI over MCP Apps (implemented)

Some changes should not be made by the model at all, only proposed by it.
Granting privileges is the first: "give bob SELECT on main.sales.orders" is a
request, and the user should see and confirm exactly what is granted to whom.
So `uc mcp serve` answers with a form instead of a write.

- **A2UI for the form.** [A2UI](https://a2ui.org) (v0.9) describes a UI as
  JSON messages over a locked component catalog: `createSurface`,
  `updateComponents` (components bound to paths in a data model), and
  `updateDataModel`. Our catalog
  (`https://open-lakehouse.dev/a2ui/uc/v1/catalog.json`) has one component,
  `UcGrantForm`, which renders the Permissions tab's `GrantForm`
  (`@open-lakehouse/unity-catalog-ui/grants`). The agent never composes UI. It
  calls `open_grant_form`, and the server emits the surface, prefilled.
- **An MCP App to render it.** Hosts that support the MCP Apps extension
  (`io.modelcontextprotocol/ui`: Claude Desktop/claude.ai, MCP Inspector, and
  others) render a `ui://` resource in a sandboxed iframe next to a tool's
  result. `ui://uc/grants` is the A2UI React renderer plus our catalog, built
  by `node/uc-a2ui` into one inlined HTML file that `uc` embeds. The A2UI
  messages ride in the tool result as an embedded `application/a2ui+json`
  resource with `audience: ["user"]`, so their JSON stays out of the model's
  context.
- **Only MCP Apps clients get the A2UI.** The embedded resource, and the
  result's `_meta.ui.resourceUri`, go only to a client that declared
  `io.modelcontextprotocol/ui`. Every client gets the text and
  `structuredContent` (`status: "awaiting_user"`, the prefill, any
  candidates), so an agent can still say the form is open and that nothing
  is granted until the user submits it. An agent can't render the resource,
  and hosts that flatten results ignore `audience` and put the JSON in the
  agent's context. Some agent hosts also fail on embedded resources outright
  (Omnigent's result formatter). The app stays discoverable from the tool's
  own `_meta.ui` in `tools/list`. A host that renders apps for an agent's
  session re-runs the read-only `open_grant_form` on its own connection,
  with the extension, for the full result. The same rule covers
  `apply_grants`' outcome resource, though only app clients can call it.
- **The write needs a click.** Submitting the form calls `apply_grants`,
  which carries `_meta.ui.visibility: ["app"]`: the host keeps it from the
  model. `uc` only lists `apply_grants` and `whoami`, and only accepts calls to
  them, when the client declared the MCP Apps extension, so a host without it
  never offers the write to the model. The outcome goes back into the form
  (`/status`, `/assignments`) and to the agent through `ui/update-model-context`.
  A model-callable grant tool would sit behind `--allow-writes` (§2). It is
  not built.
- **No token in the iframe.** The app has no network. Every call goes through
  `uc`'s tools as the signed-in user, and the server's Cedar policy still
  decides (OWNER or MANAGE on the securable).
- **Principals from the directory.** `open_grant_form` looks the agent's
  principal hint up in the server's read-only SCIM API (users by user name or
  display name, groups and service principals by name):
  - An exact match, or the only match, is prefilled.
  - Several matches are offered in the form, and listed for the agent.
  - No match, or no SCIM API (404), keeps the hint as typed, and the server
    checks it on submit.

  The form's principal field autocompletes through an `IdentityHost` over the
  `search_principals` tool, the MCP twin of `createScimIdentityHost`.

Tools: `get_grants`, `open_grant_form`, `search_principals` (all read-only),
plus the app-only `apply_grants` and `whoami`. Run it with
`{ "command": "uc", "args": ["mcp", "serve"] }`, or over Streamable HTTP
(§1), where each connection is gated on its own capabilities. Build with
`--features mcp`, and refresh the embedded app with `just build mcp-app`.

Follow-ups: revoke and transfer-ownership surfaces in the same catalog; reuse
the catalog in our own app's agent pane, rendering A2UI natively without the
MCP App iframe; and validate privilege applicability on the server
(mangrove#353), which today the form does for the server.

## Validation lenses

The five agent-CLI lenses (descriptions, discovery, schema awareness,
governance audits, profiling) apply to the catalog tools unchanged. Function
tools add three:

1. An agent calls a governed business function (`main.tools.get_ltv(customer_id)`)
   and gets a typed result, with no setup beyond `--functions main.tools`.
2. A function author edits a trusted function: the tool disables itself until
   the user re-approves the new body.
3. A function tries to read `~/.ssh`, open a socket, or read the catalog
   token: each attempt is denied and recorded in the audit log.

## Staged follow-ups

- More A2UI surfaces (§8): revoke, ownership transfer, tag editing.
- Inbound auth on the Streamable HTTP transport (§1): OAuth (MCP
  authorization spec) and per-request bearer forwarding, for a shared
  deployment next to `olai-uc-server`.
- The `wasm` sandbox backend.
- Egress allow-list proxy for functions that need the network.
- `READS_SQL_DATA` SQL functions and table-valued functions (with `preview`).
- Server-side function execution through the same `function-runtime` crate.
- MCP prompts (e.g. "document this schema") built from the validation lenses.

## Explicitly out of scope (for now)

- A generic "run arbitrary Python/SQL" tool. Only catalog-registered, trusted
  functions run.
- A free-form SQL tool over catalog tables (revisit with the read stack).
- A native Windows sandbox. Use the `container` backend.

## Open questions

1. **Where Python dependencies live in OSS UC.** The OSS model has
   `routine_dependencies` (data dependencies) and free-form `properties`.
   Databricks uses an `ENVIRONMENT (dependencies = …, environment_version = …)`
   clause. We should pick one representation, ideally shared with
   unitycatalog-ai, and possibly promote it to a proto field.
2. **Who checks `EXECUTE`.** Today any caller who can read a function's
   definition can run it locally. Should the server expose a
   "can I execute this?" check (or enforce it by redacting `routine_definition`),
   so local execution respects the same grant as remote execution?
3. **Default with no allowlist.** We propose no function tools. The
   alternative, all functions in a configured default schema, is easier to
   start with but breaks the "nothing runs unless named" rule.
4. **Python version.** Pin per function (a property) or per server
   (`--python`)? Databricks ties it to an environment version.

## Sources

- Databricks managed MCP servers — https://docs.databricks.com/aws/en/generative-ai/mcp/managed-mcp
- unitycatalog-ai — https://docs.unitycatalog.io/ai/quickstart/ and
  https://github.com/unitycatalog/unitycatalog/tree/main/ai/core
- Apache Polaris MCP server — https://github.com/apache/polaris-tools/blob/main/mcp-server/README.md
- Apache Gravitino MCP server — https://gravitino.apache.org/docs/next/gravitino-mcp-server/
- DataHub MCP — https://github.com/datahub-project/datahub/blob/master/docs/features/feature-guides/mcp.md
- Rust MCP SDK (`rmcp`) — https://github.com/modelcontextprotocol/rust-sdk
- uv — https://docs.astral.sh/uv/
- OpenAI Codex CLI sandboxing — https://github.com/openai/codex
- Anthropic sandbox-runtime — https://github.com/anthropic-experimental/sandbox-runtime
- Landlock — https://docs.rs/landlock ; wasmtime resource limits — https://docs.wasmtime.dev/
- Pyodide — https://pyodide.org/

This document was AI-assisted by Isaac.
