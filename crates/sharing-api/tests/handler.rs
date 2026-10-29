//! Handler integration tests: drive the discovery + asset surface through the
//! generated handler traits over the in-memory backend.

use open_sharing_api::catalog::AssetKind;
use open_sharing_api::codegen::sharing::SharingHandler;
use open_sharing_api::codegen::sharing_skill::SharingSkillHandler;
use open_sharing_api::codegen::sharing_volume::SharingVolumeHandler;
use open_sharing_api::testing::{FixtureObject, FixtureShare, in_memory_service};
use open_sharing_api::{Error, SharingQueryHandler, SharingService};
use open_sharing_common::models::open_sharing::v1::*;

fn backend() -> SharingService<()> {
    in_memory_service(vec![FixtureShare {
        name: "share1".to_string(),
        id: Some("share1-id".to_string()),
        comment: Some("a shared dataset".to_string()),
        objects: vec![
            FixtureObject::table("schema1", "table1", "s3://bucket/table1"),
            FixtureObject::new(AssetKind::Volume, "schema1", "vol1", "s3://bucket/vol1"),
            FixtureObject::new(
                AssetKind::AgentSkill,
                "schema1",
                "skill1",
                "s3://bucket/skill1",
            ),
            // A schema that only holds a volume must still be listed.
            FixtureObject::new(AssetKind::Volume, "schema2", "vol2", "s3://bucket/vol2"),
        ],
    }])
}

#[tokio::test]
async fn lists_shares() {
    let b = backend();
    let resp = SharingHandler::<()>::list_shares(&b, ListSharesRequest::default(), ())
        .await
        .unwrap();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.items[0].name, "share1");
    assert_eq!(resp.items[0].comment.as_deref(), Some("a shared dataset"));
}

#[tokio::test]
async fn gets_share() {
    let b = backend();
    let share = SharingHandler::<()>::get_share(
        &b,
        GetShareRequest {
            name: "share1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(share.name, "share1");
    assert_eq!(share.id.as_deref(), Some("share1-id"));
}

#[tokio::test]
async fn lists_schemas_and_tables() {
    let b = backend();
    let schemas = SharingHandler::<()>::list_schemas(
        &b,
        ListSchemasRequest {
            share: "share1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    let names: Vec<_> = schemas.items.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["schema1", "schema2"]);

    let tables = SharingHandler::<()>::list_tables(
        &b,
        ListTablesRequest {
            share: "share1".to_string(),
            name: "schema1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(tables.items.len(), 1);
    assert_eq!(tables.items[0].name, "table1");
    assert_eq!(tables.items[0].schema, "schema1");
    assert_eq!(tables.items[0].share, "share1");

    let all = SharingHandler::<()>::list_all_tables(
        &b,
        ListAllTablesRequest {
            name: "share1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(all.items.len(), 1);
    assert_eq!(all.items[0].name, "table1");
}

#[tokio::test]
async fn lists_and_vends_volumes() {
    let b = backend();
    let vols = SharingVolumeHandler::<()>::list_volumes(
        &b,
        ListVolumesRequest {
            share: "share1".to_string(),
            schema: "schema1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(vols.items.len(), 1);
    assert_eq!(vols.items[0].name, "vol1");

    let creds = SharingVolumeHandler::<()>::generate_temporary_volume_credentials(
        &b,
        GenerateTemporaryVolumeCredentialsRequest {
            share: "share1".to_string(),
            schema: "schema1".to_string(),
            name: "vol1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(creds.url.as_deref(), Some("s3://bucket/vol1"));
    assert!(creds.credentials.is_some());
}

#[tokio::test]
async fn lists_skills() {
    let b = backend();
    let skills = SharingSkillHandler::<()>::list_skills(
        &b,
        ListSkillsRequest {
            share: "share1".to_string(),
            schema: "schema1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(skills.items.len(), 1);
    assert_eq!(skills.items[0].name, "skill1");
}

#[tokio::test]
async fn missing_share_is_not_found() {
    let b = backend();
    let err = SharingHandler::<()>::get_share(
        &b,
        GetShareRequest {
            name: "nope".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, Error::NotFound));
}

#[tokio::test]
async fn gap_endpoints_are_not_implemented() {
    let b = backend();
    let changes = SharingQueryHandler::<()>::get_table_changes(
        &b,
        QueryTableRequest {
            share: "share1".to_string(),
            schema: "schema1".to_string(),
            name: "table1".to_string(),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap_err();
    assert!(matches!(changes, Error::NotImplemented(_)));

    let poll = SharingQueryHandler::<()>::poll_query(&b, "q1".to_string(), ())
        .await
        .unwrap_err();
    assert!(matches!(poll, Error::NotImplemented(_)));
}

#[tokio::test]
async fn list_schemas_paginates() {
    let b = backend();
    let first = SharingHandler::<()>::list_schemas(
        &b,
        ListSchemasRequest {
            share: "share1".to_string(),
            max_results: Some(1),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(first.items.len(), 1);
    assert_eq!(first.items[0].name, "schema1");
    let token = first.next_page_token.expect("a second page");

    let second = SharingHandler::<()>::list_schemas(
        &b,
        ListSchemasRequest {
            share: "share1".to_string(),
            max_results: Some(1),
            page_token: Some(token),
            ..Default::default()
        },
        (),
    )
    .await
    .unwrap();
    assert_eq!(second.items.len(), 1);
    assert_eq!(second.items[0].name, "schema2");
    assert_eq!(second.next_page_token, None);
}
