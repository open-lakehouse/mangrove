//! Query-path integration tests: serve a real Delta table (the `basic_partitioned`
//! dat fixture, see `tests/data/README.md`) through the kernel-backed Delta
//! format, from the handler down and through the router.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use url::Url;

use open_sharing_api::testing::{FixtureObject, FixtureShare, in_memory_service};
use open_sharing_api::{Error, SharingQueryHandler, SharingService, open_sharing_router};
use open_sharing_common::models::open_sharing::v1::*;
use open_sharing_common::models::{
    MetadataResponseData, ParquetFile, ProtocolResponseData, QueryResponseLine,
};

const SHARE: &str = "share1";
const SCHEMA: &str = "schema1";
const TABLE: &str = "letters";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/basic_partitioned")
}

fn table_url(dir: &Path) -> String {
    Url::from_directory_path(dir.canonicalize().unwrap())
        .unwrap()
        .to_string()
}

fn service_with(object: FixtureObject) -> SharingService<()> {
    in_memory_service(vec![FixtureShare {
        name: SHARE.to_string(),
        id: Some("share1-id".to_string()),
        objects: vec![object],
        ..Default::default()
    }])
}

fn service() -> SharingService<()> {
    service_with(FixtureObject::table(
        SCHEMA,
        TABLE,
        &table_url(&fixture_dir()),
    ))
}

fn query(request: QueryTableRequest) -> QueryTableRequest {
    QueryTableRequest {
        share: SHARE.to_string(),
        schema: SCHEMA.to_string(),
        name: TABLE.to_string(),
        ..request
    }
}

/// Split a query response into its protocol line, metadata line and files,
/// checking the protocol + metadata lines come first, in that order.
fn parse(body: &[u8]) -> (ProtocolResponseData, MetadataResponseData, Vec<ParquetFile>) {
    let mut lines = body
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice::<QueryResponseLine>(l).unwrap());
    let Some(QueryResponseLine::Protocol(protocol)) = lines.next() else {
        panic!("first line must be the protocol");
    };
    let Some(QueryResponseLine::MetaData(metadata)) = lines.next() else {
        panic!("second line must be the metadata");
    };
    let files = lines
        .map(|line| match line {
            QueryResponseLine::File(file) => file,
            other => panic!("unexpected line after the header: {other:?}"),
        })
        .collect();
    (protocol, metadata, files)
}

fn letters(files: &[ParquetFile]) -> Vec<String> {
    let mut letters: Vec<_> = files
        .iter()
        .map(|f| f.partition_values["letter"].clone())
        .collect();
    letters.sort();
    letters
}

fn letter_equals(letter: &str) -> JsonPredicate {
    serde_json::from_value(serde_json::json!({
        "op": "equal",
        "children": [
            {"op": "column", "name": "letter", "valueType": "string"},
            {"op": "literal", "value": letter, "valueType": "string"}
        ]
    }))
    .unwrap()
}

#[tokio::test]
async fn version_is_latest() {
    let resp = SharingQueryHandler::<()>::get_table_version(
        &service(),
        GetTableVersionRequest {
            share: SHARE.to_string(),
            schema: SCHEMA.to_string(),
            name: TABLE.to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(resp.version, 1);
}

#[tokio::test]
async fn metadata_is_protocol_then_metadata() {
    let body = SharingQueryHandler::<()>::get_table_metadata(
        &service(),
        GetTableMetadataRequest {
            share: SHARE.to_string(),
            schema: SCHEMA.to_string(),
            name: TABLE.to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(body.version, 1);
    let (protocol, metadata, files) = parse(&body.body);
    assert_eq!(protocol.min_reader_version(), 1);
    assert_eq!(metadata.partition_columns(), ["letter".to_string()]);
    assert!(files.is_empty());
}

#[tokio::test]
async fn query_lists_latest_files() {
    let body = SharingQueryHandler::<()>::query_table(&service(), query(Default::default()), ())
        .await
        .unwrap();
    let (_, metadata, files) = parse(&body.body);
    assert_eq!(letters(&files), ["a", "a", "b", "c", "e", "f"]);

    let root = table_url(&fixture_dir());
    for file in &files {
        assert!(file.url.starts_with(&root), "unexpected url {}", file.url);
        assert_eq!(file.stats.as_deref(), Some("{\"numRecords\":1}"));
        assert_eq!(file.version, None);
        assert!(file.size > 0);
    }
    let mut ids: Vec<_> = files.iter().map(|f| f.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), files.len(), "file ids must be unique");

    let MetadataResponseData::ParquetMetadata(metadata) = metadata else {
        panic!("expected parquet-format metadata");
    };
    assert_eq!(metadata.version, None);
}

#[tokio::test]
async fn query_at_version() {
    let body = SharingQueryHandler::<()>::query_table(
        &service(),
        query(QueryTableRequest {
            version: Some(0),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap();
    let (_, metadata, files) = parse(&body.body);
    assert_eq!(letters(&files), ["a", "b", "c"]);
    assert!(files.iter().all(|f| f.version == Some(0)));
    let MetadataResponseData::ParquetMetadata(metadata) = metadata else {
        panic!("expected parquet-format metadata");
    };
    assert_eq!(metadata.version, Some(0));
}

#[tokio::test]
async fn query_prunes_with_predicate_hints() {
    let body = SharingQueryHandler::<()>::query_table(
        &service(),
        query(QueryTableRequest {
            json_predicate_hints: letter_equals("a").into(),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap();
    let (_, _, files) = parse(&body.body);
    assert_eq!(letters(&files), ["a", "a"]);
}

#[tokio::test]
async fn query_honors_limit_hint() {
    let body = SharingQueryHandler::<()>::query_table(
        &service(),
        query(QueryTableRequest {
            limit_hint: Some(2),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap();
    let (_, _, files) = parse(&body.body);
    // Every file holds one row, so two files cover the hint.
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn query_rejects_invalid_versions() {
    let err = SharingQueryHandler::<()>::query_table(
        &service(),
        query(QueryTableRequest {
            version: Some(7),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");

    let err = SharingQueryHandler::<()>::query_table(
        &service(),
        query(QueryTableRequest {
            version: Some(0),
            timestamp: Some("2025-01-01T00:00:00Z".to_string()),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");
}

#[tokio::test]
async fn query_rejects_unshared_history() {
    let mut table = FixtureObject::table(SCHEMA, TABLE, &table_url(&fixture_dir()));
    table.history_shared = false;
    let service = service_with(table);

    let err = SharingQueryHandler::<()>::query_table(
        &service,
        query(QueryTableRequest {
            version: Some(0),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");

    // The latest version is still readable.
    SharingQueryHandler::<()>::query_table(&service, query(Default::default()), ())
        .await
        .unwrap();
}

#[tokio::test]
async fn query_enforces_start_version() {
    let mut table = FixtureObject::table(SCHEMA, TABLE, &table_url(&fixture_dir()));
    table.start_version = Some(1);
    let service = service_with(table);

    let err = SharingQueryHandler::<()>::query_table(
        &service,
        query(QueryTableRequest {
            version: Some(0),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");

    let body = SharingQueryHandler::<()>::query_table(
        &service,
        query(QueryTableRequest {
            version: Some(1),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap();
    assert_eq!(parse(&body.body).2.len(), 6);
}

/// Copy the fixture and pin its commit times: the table has no in-commit
/// timestamps, so history is resolved from commit-file modification times, which
/// a git checkout does not preserve.
fn table_with_commit_times(commit_times: [SystemTime; 2]) -> tempfile::TempDir {
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).unwrap();
            }
        }
    }
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture_dir(), dir.path());
    for (version, time) in commit_times.into_iter().enumerate() {
        let commit = dir.path().join(format!("_delta_log/{version:020}.json"));
        std::fs::File::options()
            .write(true)
            .open(commit)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }
    dir
}

#[tokio::test]
async fn query_and_version_by_timestamp() {
    // 2001-09-09T01:46:40Z and one minute later.
    let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let t1 = t0 + Duration::from_secs(60);
    let dir = table_with_commit_times([t0, t1]);
    let service = service_with(FixtureObject::table(SCHEMA, TABLE, &table_url(dir.path())));

    // As of 30s after the first commit, the table is at version 0.
    let body = SharingQueryHandler::<()>::query_table(
        &service,
        query(QueryTableRequest {
            timestamp: Some("2001-09-09T01:47:10Z".to_string()),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap();
    let (_, _, files) = parse(&body.body);
    assert_eq!(letters(&files), ["a", "b", "c"]);
    assert!(files.iter().all(|f| f.version == Some(0)));
    assert!(files.iter().all(|f| f.timestamp == Some(1_000_000_000_000)));

    // The earliest version at or after that instant is version 1.
    let version = SharingQueryHandler::<()>::get_table_version(
        &service,
        GetTableVersionRequest {
            share: SHARE.to_string(),
            schema: SCHEMA.to_string(),
            name: TABLE.to_string(),
            starting_timestamp: Some("2001-09-09T01:47:10Z".to_string()),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(version.version, 1);

    // Nothing was committed before the first commit.
    let err = SharingQueryHandler::<()>::query_table(
        &service,
        query(QueryTableRequest {
            timestamp: Some("2000-01-01T00:00:00Z".to_string()),
            ..Default::default()
        }),
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");
}

#[tokio::test]
async fn router_query_accepts_string_encoded_predicate_hints() {
    // Delta Sharing clients send `jsonPredicateHints` as a JSON-encoded string.
    let hint = serde_json::to_string(&letter_equals("e")).unwrap();
    let body = serde_json::json!({ "jsonPredicateHints": hint }).to_string();
    let resp = open_sharing_router::<SharingService<()>, ()>(service())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/shares/{SHARE}/schemas/{SCHEMA}/tables/{TABLE}/query"
                ))
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()["content-type"],
        "application/x-ndjson; charset=utf-8"
    );
    assert_eq!(resp.headers()["delta-table-version"], "1");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let (_, _, files) = parse(&bytes);
    assert_eq!(letters(&files), ["e"]);
}
