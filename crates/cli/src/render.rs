//! Format-aware rendering for `uc client` output.
//!
//! The Unity Catalog client returns plain, serde-serializable prost structs —
//! the *render model*. This module is the *renderer*: it turns any such model
//! into JSON, a styled table, or plain tab-separated text, picking the mode
//! from the `--output` flag and whether stdout is a terminal.
//!
//! The client layer never returns pre-formatted strings; every command routes
//! its result through [`render_list`] / [`render_one`].

use std::io::IsTerminal;

use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use console::{Emoji, style};
use serde::Serialize;

use crate::error::Result;

/// User-selectable output format (the `--output/-o` flag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Table when stdout is a terminal, JSON otherwise.
    #[default]
    Auto,
    /// Pretty-printed JSON (matches the UC REST API shape; the stable wire
    /// contract for scripts).
    Json,
    /// Enriched, pruned JSON tuned for LLM agents (a versioned envelope with
    /// high-signal fields and `_next` hints; use `--raw` for the full shape).
    Agent,
    /// Styled, boxed table.
    Table,
    /// Header line plus tab-separated rows; no color or box drawing.
    Plain,
}

/// The format actually used to render, after `Auto` has been resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFormat {
    Json,
    Agent,
    Table,
    Plain,
}

impl OutputFormat {
    /// Collapse `Auto` based on whether stdout is a terminal: TTY → table,
    /// piped/redirected → JSON (scriptable by default).
    pub fn resolve(self) -> ResolvedFormat {
        match self {
            OutputFormat::Json => ResolvedFormat::Json,
            OutputFormat::Agent => ResolvedFormat::Agent,
            OutputFormat::Table => ResolvedFormat::Table,
            OutputFormat::Plain => ResolvedFormat::Plain,
            OutputFormat::Auto => {
                if std::io::stdout().is_terminal() {
                    ResolvedFormat::Table
                } else {
                    ResolvedFormat::Json
                }
            }
        }
    }
}

/// Context threaded into agent rendering: controls token-efficiency knobs.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderCtx {
    /// When set, agent mode keeps the full wire fields instead of pruning to
    /// the high-signal envelope. Wired from the global `--raw` flag.
    pub raw: bool,
}

/// A resource that can be rendered as a row in a table / plain listing.
///
/// Implemented for the UC model structs (`Catalog`, `Schema`, ...). Keeps
/// [`render_list`] / [`render_one`] generic so every command shares one code
/// path regardless of resource type.
pub trait TableView {
    /// Column headers, in row order.
    fn headers() -> Vec<&'static str>;
    /// One value per header, in the same order.
    fn row(&self) -> Vec<String>;
}

/// A resource that can be reshaped into the agent envelope.
///
/// `json()` is the faithful wire shape (a stable contract for scripts, reused
/// from [`Serialize`]); `agent()` is the enriched, pruned view an LLM agent
/// reasons over. The default `agent()` falls back to the wire shape, so a type
/// only overrides it when pruning or `_next` hints add value. `--raw`
/// ([`RenderCtx::raw`]) forces the wire shape even in agent mode.
pub trait Render: Serialize {
    /// The high-signal, token-efficient agent view. Defaults to the wire shape.
    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// The `kind` label used in the agent envelope (`catalog`, `table`, ...).
    fn kind_label() -> &'static str;
}

/// Wrap a rendered agent value (or list of values) in the versioned envelope.
fn agent_envelope(kind: &str, count: Option<usize>, body: serde_json::Value) -> serde_json::Value {
    let mut env = serde_json::Map::new();
    env.insert("_v".into(), serde_json::json!(1));
    env.insert("kind".into(), serde_json::json!(kind));
    if let Some(n) = count {
        env.insert("count".into(), serde_json::json!(n));
    }
    env.insert("data".into(), body);
    serde_json::Value::Object(env)
}

fn print_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Render a collection of items in the resolved format.
pub fn render_list<T>(items: &[T], fmt: ResolvedFormat, ctx: RenderCtx) -> Result<()>
where
    T: Render + TableView,
{
    match fmt {
        ResolvedFormat::Json => print_json(&serde_json::to_value(items)?)?,
        ResolvedFormat::Agent => {
            let body = if ctx.raw {
                serde_json::to_value(items)?
            } else {
                serde_json::Value::Array(items.iter().map(|i| i.agent(ctx)).collect())
            };
            print_json(&agent_envelope(T::kind_label(), Some(items.len()), body))?;
        }
        ResolvedFormat::Table => print_table(T::headers(), items.iter().map(TableView::row)),
        ResolvedFormat::Plain => print_plain(T::headers(), items.iter().map(TableView::row)),
    }
    Ok(())
}

/// Render a single item in the resolved format.
pub fn render_one<T>(item: &T, fmt: ResolvedFormat, ctx: RenderCtx) -> Result<()>
where
    T: Render + TableView,
{
    match fmt {
        ResolvedFormat::Json => print_json(&serde_json::to_value(item)?)?,
        ResolvedFormat::Agent => {
            let body = if ctx.raw {
                serde_json::to_value(item)?
            } else {
                item.agent(ctx)
            };
            print_json(&agent_envelope(T::kind_label(), None, body))?;
        }
        ResolvedFormat::Table => print_table(T::headers(), std::iter::once(item.row())),
        ResolvedFormat::Plain => print_plain(T::headers(), std::iter::once(item.row())),
    }
    Ok(())
}

fn print_table(headers: Vec<&'static str>, rows: impl Iterator<Item = Vec<String>>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).add_attribute(comfy_table::Attribute::Bold)),
        );

    let mut count = 0;
    for row in rows {
        count += 1;
        table.add_row(
            row.into_iter()
                .enumerate()
                // Cycle the first columns through colors so the primary
                // identifier stands out, mirroring the old table styling.
                .map(|(i, value)| Cell::new(value).fg(column_color(i))),
        );
    }

    if count == 0 {
        status::info("No results");
        return;
    }
    println!("{table}");
}

fn print_plain(headers: Vec<&'static str>, rows: impl Iterator<Item = Vec<String>>) {
    println!("{}", headers.join("\t"));
    for row in rows {
        println!("{}", row.join("\t"));
    }
}

fn column_color(index: usize) -> Color {
    const PALETTE: [Color; 4] = [Color::Cyan, Color::Yellow, Color::Green, Color::Blue];
    PALETTE[index % PALETTE.len()]
}

/// Status messages for command side-effects (create/delete) and errors.
///
/// `console::style` already honors `NO_COLOR` / `CLICOLOR` and strips styling
/// when stdout is not a terminal, so these are safe to call unconditionally.
///
/// A process-wide *quiet* switch ([`set_quiet`]) suppresses the informational
/// channels (success/info/warning) for scripted and agent use; `error` is never
/// suppressed, since a caller silencing chatter still needs the failure reason.
pub mod status {
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::{Emoji, style};

    static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "✓ ");
    static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
    #[allow(dead_code)] // used by the server-start summary (Phase 2)
    static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "[!] ");
    static INFO: Emoji<'_, '_> = Emoji("ℹ️ ", "[i] ");

    static QUIET: AtomicBool = AtomicBool::new(false);

    /// Enable/disable suppression of informational status messages. Set once
    /// from the global `--quiet` flag at startup.
    pub fn set_quiet(quiet: bool) {
        QUIET.store(quiet, Ordering::Relaxed);
    }

    fn quiet() -> bool {
        QUIET.load(Ordering::Relaxed)
    }

    /// Print a success message (to stdout). Suppressed when quiet.
    pub fn success(message: &str) {
        if quiet() {
            return;
        }
        println!("{}{}", CHECKMARK, style(message).green());
    }

    /// Print an error message (to stderr). Never suppressed.
    pub fn error(message: &str) {
        eprintln!("{}{}", CROSS_MARK, style(message).red());
    }

    /// Print a warning message (to stderr). Suppressed when quiet.
    #[allow(dead_code)] // used by the server-start summary (Phase 2)
    pub fn warning(message: &str) {
        if quiet() {
            return;
        }
        eprintln!("{}{}", WARNING, style(message).yellow());
    }

    /// Print an informational message (to stdout). Suppressed when quiet.
    pub fn info(message: &str) {
        if quiet() {
            return;
        }
        println!("{}{}", INFO, style(message).blue());
    }
}

/// Report a CLI error in the selected output mode.
///
/// In `json`/`agent` modes (and their `auto`-resolved equivalent when piped) the
/// error is a structured object on stderr — `{"error": "...", "kind": "..."}` —
/// so an agent can branch on `kind` without parsing prose. In human modes it is
/// a styled one-liner. This is independent of `--quiet`: an error is always
/// reported.
pub fn report_error(err: &crate::error::Error, output: OutputFormat, _quiet: bool) {
    let structured = matches!(
        output.resolve(),
        ResolvedFormat::Json | ResolvedFormat::Agent
    );
    if structured {
        let body = serde_json::json!({
            "error": err.to_string(),
            "kind": err.kind().as_str(),
        });
        eprintln!("{}", serde_json::to_string(&body).unwrap_or_default());
    } else {
        status::error(&err.to_string());
    }
}

/// Print the static capabilities primer (`uc schema`).
///
/// Emits the identifier grammar, entity kinds, output modes, the agent-envelope
/// shape, and a commands map (question + example per verb) so an agent can prime
/// once instead of re-deriving the surface from `--help`. No server call. In
/// table/plain modes it prints the same JSON (the primer is inherently
/// structured).
pub fn print_schema(_fmt: ResolvedFormat) {
    let doc = serde_json::json!({
        "_v": 1,
        "about": "uc is a CLI for a Unity Catalog server. It manages securables \
                  (catalogs, schemas, tables, volumes, functions) and renders \
                  results as a table, faithful JSON, or an agent envelope.",
        "identifier_grammar": {
            "form": "<catalog>.<schema>.<table|volume|function>",
            "examples": ["main", "main.default", "main.default.orders"],
        },
        "entity_kinds": {
            "catalog": "top-level namespace containing schemas",
            "schema": "namespace within a catalog containing tables/volumes/functions",
            "table": "a Delta/managed/external table with typed columns",
            "volume": "a storage location for non-tabular files",
            "function": "a registered UDF",
        },
        "output_modes": {
            "auto": "table on a TTY, JSON when piped (default)",
            "json": "faithful wire message (stable; for scripts)",
            "agent": "pruned envelope: {_v, kind, count?, data} with high-signal \
                      fields and `_next` hints; use --raw for the full shape",
            "table": "human-readable boxed table",
            "plain": "header + tab-separated rows",
        },
        "agent_envelope": {
            "_v": "envelope version (currently 1)",
            "kind": "the entity kind of `data`",
            "count": "present on list results",
            "data": "the pruned entity (or array of entities)",
            "_next": "suggested runnable follow-up `uc …` commands",
        },
        "errors": {
            "shape": "{\"error\": string, \"kind\": string} on stderr in json/agent modes",
            "kinds": ["not_found", "auth", "already_exists", "usage", "other"],
            "exit_codes": {"ok": 0, "other": 1, "usage": 2, "not_found": 3, "auth": 4},
        },
        "commands": {
            "uc schema": {"q": "what can this CLI do?", "eg": "uc -o agent schema"},
            "uc client catalogs list": {"q": "what catalogs exist?", "eg": "uc -o agent client catalogs list"},
            "uc client schemas list <catalog>": {"q": "what schemas are in a catalog?", "eg": "uc -o agent client schemas list main"},
            "uc client tables list <catalog> <schema>": {"q": "what tables are here?", "eg": "uc -o agent client tables list main default"},
            "uc client tables get <catalog> <schema> <table>": {"q": "what is this table's schema/columns?", "eg": "uc -o agent client tables get main default orders"},
            "uc client volumes list <catalog> <schema>": {"q": "what volumes are here?", "eg": "uc -o agent client volumes list main default"},
            "uc client functions list <catalog> <schema>": {"q": "what functions are here?", "eg": "uc -o agent client functions list main default"},
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into())
    );
}

// ---------------------------------------------------------------------------
// TableView impls for the UC model structs.
// ---------------------------------------------------------------------------

const NONE: &str = "-";

impl TableView for unitycatalog_common::Catalog {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "ID", "Comment", "Storage Root", "Properties"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.id.clone().unwrap_or_else(|| NONE.into()),
            self.comment.clone().unwrap_or_else(|| NONE.into()),
            self.storage_root.clone().unwrap_or_else(|| NONE.into()),
            if self.properties.is_empty() {
                NONE.into()
            } else {
                format!("{} properties", self.properties.len())
            },
        ]
    }
}

impl TableView for unitycatalog_common::Schema {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Full Name", "Catalog", "Comment"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.full_name.clone(),
            self.catalog_name.clone(),
            self.comment.clone().unwrap_or_else(|| NONE.into()),
        ]
    }
}

impl TableView for unitycatalog_common::Table {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Full Name", "Type", "Format", "Columns"]
    }

    fn row(&self) -> Vec<String> {
        use buffa::Enumeration;
        let table_type = self
            .table_type
            .as_known()
            .map(|t| t.proto_name().to_string())
            .unwrap_or_else(|| NONE.into());
        let format = self
            .data_source_format
            .as_known()
            .map(|f| f.proto_name().to_string())
            .unwrap_or_else(|| NONE.into());
        vec![
            self.name.clone(),
            self.full_name.clone(),
            table_type,
            format,
            self.columns.len().to_string(),
        ]
    }
}

impl TableView for unitycatalog_common::Volume {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Full Name", "Type", "Storage Location"]
    }

    fn row(&self) -> Vec<String> {
        use buffa::Enumeration;
        let volume_type = self
            .volume_type
            .as_known()
            .map(|t| t.proto_name().to_string())
            .unwrap_or_else(|| NONE.into());
        vec![
            self.name.clone(),
            self.full_name.clone(),
            volume_type,
            self.storage_location.clone(),
        ]
    }
}

impl TableView for unitycatalog_common::Function {
    fn headers() -> Vec<&'static str> {
        vec!["Name", "Full Name", "Data Type", "Comment"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.full_name.clone(),
            self.data_type.clone(),
            self.comment.clone().unwrap_or_else(|| NONE.into()),
        ]
    }
}

// ---------------------------------------------------------------------------
// Render (agent envelope) impls for the UC model structs.
//
// The agent view drops low-signal fields (UUIDs, timestamps, empty/null
// properties) and surfaces the answer an agent reasons over, with `_next`
// runnable follow-up commands. `--raw` bypasses this and emits the wire shape.
// ---------------------------------------------------------------------------

impl Render for unitycatalog_common::Catalog {
    fn kind_label() -> &'static str {
        "catalog"
    }

    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "comment": self.comment,
            "storage_root": self.storage_root,
            "property_count": self.properties.len(),
            "_next": [
                format!("uc client schemas list {}", self.name),
                format!("uc set-comment {} --comment \"…\"", self.name),
            ],
        })
    }
}

impl Render for unitycatalog_common::Schema {
    fn kind_label() -> &'static str {
        "schema"
    }

    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "full_name": self.full_name,
            "catalog": self.catalog_name,
            "comment": self.comment,
            "_next": [
                format!("uc client tables list {} {}", self.catalog_name, self.name),
            ],
        })
    }
}

impl Render for unitycatalog_common::Table {
    fn kind_label() -> &'static str {
        "table"
    }

    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        use buffa::Enumeration;
        let table_type = self
            .table_type
            .as_known()
            .map(|t| t.proto_name().to_string());
        let format = self
            .data_source_format
            .as_known()
            .map(|f| f.proto_name().to_string());
        // Columns are the high-signal payload: emit name/type/nullable/comment,
        // dropping IDs and precision/scale internals that agents rarely need.
        let columns: Vec<serde_json::Value> = self
            .columns
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "type": c.type_text,
                    "nullable": c.nullable,
                    "comment": c.comment,
                })
            })
            .collect();
        serde_json::json!({
            "name": self.name,
            "full_name": self.full_name,
            "table_type": table_type,
            "data_source_format": format,
            "comment": self.comment,
            "storage_location": self.storage_location,
            "columns": columns,
            "column_count": self.columns.len(),
            "_next": [
                format!("uc preview {}", self.full_name),
                format!("uc set-comment {} --comment \"…\"", self.full_name),
            ],
        })
    }
}

impl Render for unitycatalog_common::Volume {
    fn kind_label() -> &'static str {
        "volume"
    }

    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        use buffa::Enumeration;
        let volume_type = self
            .volume_type
            .as_known()
            .map(|t| t.proto_name().to_string());
        serde_json::json!({
            "name": self.name,
            "full_name": self.full_name,
            "volume_type": volume_type,
            "storage_location": self.storage_location,
            "comment": self.comment,
        })
    }
}

impl Render for unitycatalog_common::Function {
    fn kind_label() -> &'static str {
        "function"
    }

    fn agent(&self, _ctx: RenderCtx) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "full_name": self.full_name,
            "data_type": self.data_type,
            "comment": self.comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use unitycatalog_common::{Column, Table};

    use super::*;

    fn sample_table() -> Table {
        Table {
            name: "orders".into(),
            full_name: "main.default.orders".into(),
            comment: Some("order facts".into()),
            columns: vec![Column {
                name: "id".into(),
                type_text: "bigint".into(),
                nullable: Some(false),
                comment: Some("primary key".into()),
                // A low-signal field the agent view must drop.
                column_id: Some("col-uuid-1234".into()),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn output_format_resolves_agent() {
        assert_eq!(OutputFormat::Agent.resolve(), ResolvedFormat::Agent);
        assert_eq!(OutputFormat::Json.resolve(), ResolvedFormat::Json);
    }

    #[test]
    fn envelope_wraps_single_item() {
        let env = agent_envelope("table", None, serde_json::json!({"name": "orders"}));
        assert_eq!(env["_v"], 1);
        assert_eq!(env["kind"], "table");
        assert!(env.get("count").is_none());
        assert_eq!(env["data"]["name"], "orders");
    }

    #[test]
    fn envelope_wraps_list_with_count() {
        let env = agent_envelope("table", Some(2), serde_json::json!([{}, {}]));
        assert_eq!(env["count"], 2);
        assert!(env["data"].is_array());
    }

    #[test]
    fn table_agent_view_is_pruned_and_keeps_columns() {
        let value = sample_table().agent(RenderCtx::default());
        // High-signal fields are present.
        assert_eq!(value["full_name"], "main.default.orders");
        assert_eq!(value["column_count"], 1);
        let col = &value["columns"][0];
        assert_eq!(col["name"], "id");
        assert_eq!(col["type"], "bigint");
        assert_eq!(col["nullable"], false);
        assert_eq!(col["comment"], "primary key");
        // Low-signal column identifiers are dropped in the agent view.
        assert!(col.get("column_id").is_none());
        assert!(col.get("type_json").is_none());
        // `_next` teaches the follow-up investigation.
        let next = value["_next"].as_array().expect("_next array");
        assert!(next.iter().any(|c| {
            c.as_str()
                .is_some_and(|s| s.contains("uc set-comment main.default.orders"))
        }));
    }

    #[test]
    fn raw_flag_is_carried_in_ctx() {
        let ctx = RenderCtx { raw: true };
        assert!(ctx.raw);
    }
}
