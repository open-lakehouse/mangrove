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
    /// Pretty-printed JSON (matches the UC REST API shape).
    Json,
    /// Styled, boxed table.
    Table,
    /// Header line plus tab-separated rows; no color or box drawing.
    Plain,
}

/// The format actually used to render, after `Auto` has been resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFormat {
    Json,
    Table,
    Plain,
}

impl OutputFormat {
    /// Collapse `Auto` based on whether stdout is a terminal: TTY → table,
    /// piped/redirected → JSON (scriptable by default).
    pub fn resolve(self) -> ResolvedFormat {
        match self {
            OutputFormat::Json => ResolvedFormat::Json,
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

/// Render a collection of items in the resolved format.
pub fn render_list<T>(items: &[T], fmt: ResolvedFormat) -> Result<()>
where
    T: Serialize + TableView,
{
    match fmt {
        ResolvedFormat::Json => println!("{}", serde_json::to_string_pretty(items)?),
        ResolvedFormat::Table => print_table(T::headers(), items.iter().map(TableView::row)),
        ResolvedFormat::Plain => print_plain(T::headers(), items.iter().map(TableView::row)),
    }
    Ok(())
}

/// Render a single item in the resolved format.
pub fn render_one<T>(item: &T, fmt: ResolvedFormat) -> Result<()>
where
    T: Serialize + TableView,
{
    match fmt {
        ResolvedFormat::Json => println!("{}", serde_json::to_string_pretty(item)?),
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
pub mod status {
    use super::{Emoji, style};

    static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "✓ ");
    static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "✗ ");
    #[allow(dead_code)] // used by the server-start summary (Phase 2)
    static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "[!] ");
    static INFO: Emoji<'_, '_> = Emoji("ℹ️ ", "[i] ");

    /// Print a success message (to stdout).
    pub fn success(message: &str) {
        println!("{}{}", CHECKMARK, style(message).green());
    }

    /// Print an error message (to stderr).
    pub fn error(message: &str) {
        eprintln!("{}{}", CROSS_MARK, style(message).red());
    }

    /// Print a warning message (to stderr).
    #[allow(dead_code)] // used by the server-start summary (Phase 2)
    pub fn warning(message: &str) {
        eprintln!("{}{}", WARNING, style(message).yellow());
    }

    /// Print an informational message (to stdout).
    pub fn info(message: &str) {
        println!("{}{}", INFO, style(message).blue());
    }
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
