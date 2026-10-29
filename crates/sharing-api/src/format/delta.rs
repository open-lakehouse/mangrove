//! Serving Delta tables through `delta_kernel`.
//!
//! Files are listed straight from kernel log replay (`Scan::scan_metadata` →
//! `visit_scan_files`), with predicate hints pushed into the scan for file
//! skipping. No query engine is involved.
//!
//! Responses use the protocol's parquet response format, which can only describe
//! tables a plain parquet reader understands (`minReaderVersion` 1). Tables that
//! need reader features (column mapping, deletion vectors, …) require the delta
//! response format, which is not served yet; such tables are rejected rather than
//! served incorrectly.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use delta_kernel::history_manager::{
    CommitAt, HistoryCommitType, first_version_after, latest_version_as_of,
};
use delta_kernel::scan::state::ScanFile;
use delta_kernel::snapshot::SnapshotRef;
use delta_kernel::{Engine, Snapshot, Version};
use serde::Serialize;
use url::Url;

use open_sharing_common::models::{
    MetadataResponse, MetadataResponseData, ParquetFile, ParquetMetadata, ProtocolResponseData,
    QueryResponseLine,
};

use super::predicate::to_kernel_predicate;
use super::{QuerySpec, TableFormat, TableResponse, VersionSpec};
use crate::access::TableStorage;
use crate::catalog::SharedAsset;
use crate::error::{Error, Result};
use crate::kernel::build_engine;

/// The kernel-backed Delta [`TableFormat`]. See the module docs.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeltaFormat;

#[async_trait]
impl TableFormat for DeltaFormat {
    async fn version(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        starting_timestamp: Option<i64>,
    ) -> Result<Version> {
        let engine = build_engine(storage.store.clone());
        let root = directory(&storage.root);
        let start_version = asset.start_version;
        blocking(move || {
            let latest = Snapshot::builder_for(root.as_str()).build(engine.as_ref())?;
            let version = match starting_timestamp {
                None => latest.version(),
                Some(ts) => {
                    first_version_after(
                        &latest,
                        engine.as_ref(),
                        ts,
                        HistoryCommitType::Recreatable,
                    )
                    .map_err(|e| Error::invalid_argument(format!("startingTimestamp: {e}")))?
                    .version
                }
            };
            Ok(start_version.map_or(version, |start| version.max(start)))
        })
        .await
    }

    async fn metadata(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        at: VersionSpec,
    ) -> Result<TableResponse> {
        let engine = build_engine(storage.store.clone());
        let root = directory(&storage.root);
        let start_version = asset.start_version;
        let (snapshot, _) =
            blocking(move || resolve_snapshot(engine.as_ref(), &root, at, start_version)).await?;
        let mut out = Vec::new();
        for line in header_lines(&snapshot, at)? {
            write_line(&mut out, &line)?;
        }
        Ok(TableResponse {
            version: snapshot.version(),
            body: Bytes::from(out),
        })
    }

    async fn query(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        query: QuerySpec,
    ) -> Result<TableResponse> {
        let engine = build_engine(storage.store.clone());
        let root = directory(&storage.root);
        let start_version = asset.start_version;
        let at = query.at;
        let predicate = query.predicate.as_ref().and_then(to_kernel_predicate);
        let (snapshot, commit, files) = blocking(move || {
            let (snapshot, commit) = resolve_snapshot(engine.as_ref(), &root, at, start_version)?;
            let scan = snapshot
                .clone()
                .scan_builder()
                .with_predicate(predicate.map(Arc::new))
                .build()?;
            let mut files = Vec::new();
            for scan_metadata in scan.scan_metadata(engine.as_ref())? {
                files = scan_metadata?
                    .visit_scan_files(files, |files: &mut Vec<ScanFile>, file| files.push(file))?;
            }
            Ok((snapshot, commit, files))
        })
        .await?;

        let files = apply_limit_hint(files, query.limit_hint);
        let root = directory(&storage.root);
        let urls = files
            .iter()
            .map(|file| file_url(&root, &file.path))
            .collect::<Result<Vec<_>>>()?;
        let signed = storage.signer.sign(&urls).await?;
        if signed.len() != files.len() {
            return Err(Error::generic(
                "file signer returned a mismatched URL count",
            ));
        }

        let mut out = Vec::new();
        for line in header_lines(&snapshot, at)? {
            write_line(&mut out, &line)?;
        }
        let (version, timestamp) = match at {
            VersionSpec::Latest => (None, None),
            _ => (
                Some(snapshot.version() as i64),
                commit.map(|commit| commit.timestamp),
            ),
        };
        for ((file, url), signed) in files.into_iter().zip(&urls).zip(signed) {
            let line = QueryResponseLine::File(ParquetFile {
                url: signed.url,
                id: file_id(url),
                partition_values: file.partition_values,
                size: file.size,
                stats: file
                    .stats
                    .map(|stats| format!("{{\"numRecords\":{}}}", stats.num_records)),
                version,
                timestamp,
                expiration_timestamp: signed.expiration_timestamp,
            });
            write_line(&mut out, &line)?;
        }
        Ok(TableResponse {
            version: snapshot.version(),
            body: Bytes::from(out),
        })
    }
}

/// `url` as a directory URL (with a trailing `/`), so relative paths join under it.
fn directory(url: &Url) -> Url {
    let mut url = url.clone();
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    url
}

/// Run blocking kernel work off the async runtime.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Error::generic(format!("table reader task failed: {e}")))?
}

/// Resolve the snapshot `at` names, enforcing the provider's `start_version`.
///
/// Returns the commit the snapshot was located by, for a timestamp query.
fn resolve_snapshot(
    engine: &dyn Engine,
    root: &Url,
    at: VersionSpec,
    start_version: Option<Version>,
) -> Result<(SnapshotRef, Option<CommitAt>)> {
    let (snapshot, commit) = match at {
        VersionSpec::Latest => (Snapshot::builder_for(root.as_str()).build(engine)?, None),
        VersionSpec::Version(version) => {
            match Snapshot::builder_for(root.as_str())
                .at_version(version)
                .build(engine)
            {
                Ok(snapshot) => (snapshot, None),
                Err(err) => {
                    // Distinguish a version past the end of the table (the caller's
                    // mistake) from a failure reading it.
                    let latest = Snapshot::builder_for(root.as_str()).build(engine)?;
                    if version > latest.version() {
                        return Err(Error::invalid_argument(format!(
                            "version {version} is beyond the table's latest version {}",
                            latest.version()
                        )));
                    }
                    return Err(err.into());
                }
            }
        }
        VersionSpec::Timestamp(ts) => {
            let latest = Snapshot::builder_for(root.as_str()).build(engine)?;
            let commit = latest_version_as_of(&latest, engine, ts, HistoryCommitType::Recreatable)
                .map_err(|e| Error::invalid_argument(format!("timestamp: {e}")))?;
            let snapshot = if commit.version == latest.version() {
                latest
            } else {
                Snapshot::builder_for(root.as_str())
                    .at_version(commit.version)
                    .build(engine)?
            };
            (snapshot, Some(commit))
        }
    };
    if let Some(start) = start_version
        && snapshot.version() < start
    {
        return Err(Error::invalid_argument(format!(
            "version {} is before the earliest shared version {start}",
            snapshot.version()
        )));
    }
    Ok((snapshot, commit))
}

/// The protocol + metadata lines that open every response, in that order.
fn header_lines(snapshot: &Snapshot, at: VersionSpec) -> Result<[MetadataResponse; 2]> {
    let config = snapshot.table_configuration();
    let protocol = config.protocol();
    if protocol.min_reader_version() > 1 {
        return Err(Error::invalid_argument(format!(
            "the table requires reader version {} (reader features such as column mapping or \
             deletion vectors), which needs the delta response format; it is not served yet",
            protocol.min_reader_version()
        )));
    }
    let mut metadata: ParquetMetadata = config.metadata().try_into()?;
    if at.is_time_travel() {
        metadata.version = Some(snapshot.version() as i64);
    }
    Ok([
        MetadataResponse::Protocol(ProtocolResponseData::ParquetProtocol(protocol.into())),
        MetadataResponse::MetaData(MetadataResponseData::ParquetMetadata(metadata)),
    ])
}

/// Keep files until their known row counts cover `limit_hint`.
///
/// Files without statistics are always kept (their row count is unknown), and do
/// not count towards the limit, so the result can never undercount.
fn apply_limit_hint(files: Vec<ScanFile>, limit_hint: Option<u64>) -> Vec<ScanFile> {
    let Some(limit) = limit_hint else {
        return files;
    };
    let mut rows = 0u64;
    let mut kept = Vec::new();
    for file in files {
        if rows >= limit {
            break;
        }
        if let Some(stats) = &file.stats {
            rows = rows.saturating_add(stats.num_records);
        }
        kept.push(file);
    }
    kept
}

/// The absolute URL of a data file: log paths are relative to the table root,
/// unless they are already absolute (shallow clones, …).
fn file_url(root: &Url, path: &str) -> Result<Url> {
    match Url::parse(path) {
        Ok(url) => Ok(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => Ok(root.join(path)?),
        Err(err) => Err(err.into()),
    }
}

/// A stable file id, derived from the file's absolute URL.
fn file_id(url: &Url) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, url.as_str().as_bytes())
        .simple()
        .to_string()
}

fn write_line(out: &mut Vec<u8>, line: &impl Serialize) -> Result<()> {
    serde_json::to_writer(&mut *out, line)?;
    out.push(b'\n');
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_url_joins_relative_and_keeps_absolute_paths() {
        let root = Url::parse("s3://bucket/tables/t1/").unwrap();
        assert_eq!(
            file_url(&root, "letter=a/part-0.parquet").unwrap().as_str(),
            "s3://bucket/tables/t1/letter=a/part-0.parquet"
        );
        assert_eq!(
            file_url(&root, "s3://other/x.parquet").unwrap().as_str(),
            "s3://other/x.parquet"
        );
    }

    #[test]
    fn file_id_is_stable() {
        let url = Url::parse("s3://bucket/t/part-0.parquet").unwrap();
        assert_eq!(file_id(&url), file_id(&url));
        assert_ne!(
            file_id(&url),
            file_id(&Url::parse("s3://bucket/t/part-1.parquet").unwrap())
        );
    }
}
