//! Query execution over an opened Delta table: SQL table-reference extraction,
//! qualified-name registration, and the `open_lakehouse.query.v1` chunk framing.
//!
//! The runner contract hands us raw SQL (built by `@open-lakehouse/query` as
//! ``SELECT … FROM `catalog`.`schema`.`table` LIMIT n``). The engine extracts
//! the single table reference, resolves it against the request's optional
//! default catalog/schema (the Unity Catalog address), opens the table through
//! [`deltalake_wasm`], registers the scan under exactly the name the SQL will
//! resolve to, and executes — emitting one **self-contained** Arrow IPC stream
//! per record batch (schema + batch + EOS), per the proto contract. This
//! differs from `deltalake_wasm::query_ipc`, whose chunks form one incremental
//! stream and only parse when concatenated.
//!
//! Everything here compiles natively; the native tests drive it with an
//! [`InlineExecutor`](deltalake_core::kernel::InlineExecutor) over an in-memory
//! store — the browser execution model minus the network.

use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::SchemaRef;
use datafusion::catalog::memory::{MemoryCatalogProvider, MemorySchemaProvider};
use datafusion::execution::context::SessionContext;
use datafusion::sql::TableReference;
use datafusion::sql::parser::DFParserBuilder;
use futures::StreamExt;
use futures::stream::BoxStream;
use object_store::ObjectStore;
use url::Url;

use deltalake_core::delta_datafusion::{DeltaScanConfig, DeltaScanNext};
use deltalake_wasm::{LogSource, OpenOptions, OpenedTable, open_table_with_store};

use crate::error::{Error, Result};
use crate::resolve::DiscoveredLog;

/// The Unity Catalog address a query's table reference resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableAddress {
    pub catalog: String,
    pub schema: String,
    pub table: String,
}

impl TableAddress {
    /// `catalog.schema.table`, for messages.
    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.schema, self.table)
    }
}

/// Extract the single table reference from `sql`.
///
/// Returns the reference exactly as written (for registration) plus the Unity
/// Catalog [`TableAddress`] it denotes, filling missing qualifiers from the
/// request's optional session defaults. Rejects — as [`Error::Unsupported`] —
/// non-SELECT statements, multi-table queries, and references that stay
/// under-qualified after applying the defaults.
pub fn extract_table(
    sql: &str,
    default_catalog: Option<&str>,
    default_schema: Option<&str>,
) -> Result<(TableReference, TableAddress)> {
    use datafusion::sql::parser::Statement as DfStatement;
    use datafusion::sql::sqlparser::ast::Statement as SqlStatement;
    use datafusion::sql::sqlparser::dialect::GenericDialect;

    let mut statements = DFParserBuilder::new(sql)
        .with_dialect(&GenericDialect {})
        .build()
        .map_err(Error::DataFusion)?
        .parse_statements()
        .map_err(Error::DataFusion)?;
    if statements.len() != 1 {
        return Err(Error::unsupported(format!(
            "expected exactly one SQL statement, got {}",
            statements.len()
        )));
    }
    let statement = statements.pop_front().expect("length checked above");
    match &statement {
        DfStatement::Statement(inner) if matches!(**inner, SqlStatement::Query(_)) => {}
        _ => {
            return Err(Error::unsupported(
                "only SELECT queries run in the browser".to_string(),
            ));
        }
    }

    // `true` = normalize unquoted identifiers to lowercase, matching the
    // session default the query will execute under.
    let (mut references, _) = datafusion::sql::resolve::resolve_table_references(&statement, true)?;
    references.dedup();
    let reference = match references.as_slice() {
        [single] => single.clone(),
        [] => {
            return Err(Error::unsupported("query references no table".to_string()));
        }
        many => {
            return Err(Error::unsupported(format!(
                "query references {} tables; the in-browser engine supports one",
                many.len()
            )));
        }
    };

    let qualifier = |explicit: Option<&str>, default: Option<&str>, kind: &str| {
        explicit.or(default).map(str::to_owned).ok_or_else(|| {
            Error::unsupported(format!(
                "table reference `{reference}` has no {kind}; qualify it or set a \
                     session default"
            ))
        })
    };
    let address = TableAddress {
        catalog: qualifier(reference.catalog(), default_catalog, "catalog")?,
        schema: qualifier(reference.schema(), default_schema, "schema")?,
        table: reference.table().to_owned(),
    };
    Ok((reference, address))
}

/// Open the Delta table at `table_url` from `store`, priming the discovered log.
///
/// `executor` overrides the kernel executor (native tests force the inline
/// executor); `None` picks the target's natural choice. The snapshot is pinned
/// to the discovered log's newest version so the query sees exactly the
/// manifest that was primed.
pub async fn open_table(
    store: Arc<dyn ObjectStore>,
    table_url: &Url,
    log: DiscoveredLog,
    executor: Option<deltalake_core::kernel::ExecutorHandle>,
) -> Result<OpenedTable> {
    let opened = open_table_with_store(
        store,
        table_url,
        LogSource::Manifest(log.manifest),
        OpenOptions {
            version: Some(log.version),
            executor,
            ..OpenOptions::default()
        },
    )
    .await?;
    Ok(opened)
}

/// Register the opened table's scan under exactly the name `reference` resolves
/// to on `ctx`, creating the catalog/schema providers as needed.
///
/// Bare and partial references land in the session's default catalog/schema —
/// the same resolution `ctx.sql` applies — so the query's `FROM` clause finds
/// the scan wherever it looks.
pub fn register_table(
    ctx: &SessionContext,
    opened: &OpenedTable,
    reference: &TableReference,
) -> Result<()> {
    // Resolve bare/partial references the same way `ctx.sql` will: against the
    // session's configured default catalog and schema.
    let resolved = {
        let state = ctx.state();
        let options = &state.config().options().catalog;
        reference
            .clone()
            .resolve(&options.default_catalog, &options.default_schema)
    };

    let catalog = match ctx.catalog(resolved.catalog.as_ref()) {
        Some(existing) => existing,
        None => {
            let created = Arc::new(MemoryCatalogProvider::new());
            ctx.register_catalog(resolved.catalog.as_ref(), created.clone());
            created
        }
    };
    let schema = match catalog.schema(resolved.schema.as_ref()) {
        Some(existing) => existing,
        None => {
            let created = Arc::new(MemorySchemaProvider::new());
            catalog.register_schema(resolved.schema.as_ref(), created.clone())?;
            created
        }
    };

    let scan = DeltaScanNext::new(opened.snapshot.clone(), DeltaScanConfig::default())?;
    schema.register_table(resolved.table.to_string(), Arc::new(scan))?;
    Ok(())
}

/// One result chunk: a self-contained Arrow IPC stream plus its row count.
#[derive(Debug, Clone)]
pub struct IpcChunk {
    /// Arrow IPC stream bytes: schema message + one record batch + EOS.
    pub ipc: Vec<u8>,
    /// Rows in this chunk.
    pub num_rows: usize,
}

/// Encode `batches` (possibly none — a schema-only chunk) as one self-contained
/// Arrow IPC stream.
fn encode_chunk(schema: &SchemaRef, batches: &[&RecordBatch]) -> Result<Vec<u8>> {
    let mut writer = StreamWriter::try_new(Vec::new(), schema)?;
    for batch in batches {
        writer.write(batch)?;
    }
    Ok(writer.into_inner()?)
}

/// Execute `sql` on `ctx`, capping the result at `limit` rows when given, and
/// stream self-contained IPC chunks.
///
/// Framing per the `open_lakehouse.query.v1` contract: one chunk per record
/// batch, each independently decodable; a query with no rows yields exactly one
/// schema-only chunk.
pub async fn execute_chunks(
    ctx: &SessionContext,
    sql: &str,
    limit: Option<usize>,
) -> Result<BoxStream<'static, Result<IpcChunk>>> {
    let mut df = ctx.sql(sql).await?;
    if let Some(limit) = limit {
        df = df.limit(0, Some(limit))?;
    }
    let batches = df.execute_stream().await?;
    let schema = batches.schema();

    struct State {
        batches: datafusion::execution::SendableRecordBatchStream,
        schema: SchemaRef,
        sent_any: bool,
    }
    let stream = futures::stream::unfold(
        Some(State {
            batches,
            schema,
            sent_any: false,
        }),
        |state| async move {
            let mut state = state?;
            loop {
                return match state.batches.next().await {
                    // Skip empty batches: a chunk must carry rows (or be the
                    // one terminal schema-only chunk).
                    Some(Ok(batch)) if batch.num_rows() == 0 => continue,
                    Some(Ok(batch)) => {
                        let chunk = encode_chunk(&state.schema, &[&batch]).map(|ipc| IpcChunk {
                            ipc,
                            num_rows: batch.num_rows(),
                        });
                        state.sent_any = true;
                        Some((chunk, Some(state)))
                    }
                    Some(Err(err)) => Some((Err(err.into()), None)),
                    None if !state.sent_any => {
                        let chunk = encode_chunk(&state.schema, &[])
                            .map(|ipc| IpcChunk { ipc, num_rows: 0 });
                        Some((chunk, None))
                    }
                    None => None,
                };
            }
        },
    );
    Ok(stream.boxed())
}

// Native-only: unit tests never run on wasm32 (no test runner without
// wasm-bindgen-test), and the async ones need tokio.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn extracts_backtick_quoted_full_reference() {
        let (reference, address) =
            extract_table("SELECT * FROM `uc`.`sales`.`Orders` LIMIT 100", None, None).unwrap();
        assert_eq!(reference.table(), "Orders");
        assert_eq!(
            address,
            TableAddress {
                catalog: "uc".into(),
                schema: "sales".into(),
                table: "Orders".into(),
            }
        );
    }

    #[test]
    fn fills_defaults_for_bare_and_partial_references() {
        let (_, address) =
            extract_table("SELECT id FROM orders", Some("uc"), Some("sales")).unwrap();
        assert_eq!(address.full_name(), "uc.sales.orders");

        let (_, address) = extract_table("SELECT id FROM sales.orders", Some("uc"), None).unwrap();
        assert_eq!(address.catalog, "uc");
        assert_eq!(address.schema, "sales");

        let err = extract_table("SELECT id FROM orders", None, Some("s")).unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }

    #[test]
    fn rejects_multi_table_and_non_select() {
        let err =
            extract_table("SELECT * FROM a.b.c JOIN a.b.d ON c.id = d.id", None, None).unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        let err = extract_table("DROP TABLE a.b.c", None, None).unwrap_err();
        assert!(err.is_unsupported(), "{err}");

        // Self-joins of ONE table are fine: a single distinct reference.
        extract_table(
            "SELECT * FROM a.b.c x JOIN a.b.c y ON x.id = y.id",
            None,
            None,
        )
        .unwrap();
    }
}
