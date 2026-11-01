//! The conformance suite every [`StorageAccessStore`] backend runs.
//!
//! [`conformance`] drives a backend through the rules the [crate docs](crate) state,
//! each case on a fresh store from `make`. A backend's test calls it once:
//!
//! ```ignore
//! #[tokio::test]
//! async fn conformance() {
//!     unitycatalog_storage_access::testing::conformance(|| async { MyStore::new() }).await;
//! }
//! ```

use std::future::Future;

use olai_storage_credentials::StorageLocation;
use unitycatalog_common::models::credentials::v1::{
    AwsIamRoleConfig, AzureStorageKey, Credential, Purpose,
};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;

use crate::{
    CredentialSecrets, CredentialUpdate, Error, ExternalLocationUpdate, Page, StorageAccessStore,
};

/// Runs every conformance case, each against a fresh store from `make`.
pub async fn conformance<S, F, Fut>(mut make: F)
where
    S: StorageAccessStore,
    F: FnMut() -> Fut,
    Fut: Future<Output = S>,
{
    credential_crud(&make().await).await;
    credential_names_are_case_insensitive(&make().await).await;
    credential_pagination_and_purpose(&make().await).await;
    external_location_crud(&make().await).await;
    external_location_needs_credential(&make().await).await;
    external_locations_never_overlap(&make().await).await;
    credential_rename_follows_to_locations(&make().await).await;
    credential_in_use_cannot_be_deleted(&make().await).await;
    resolves_the_covering_location(&make().await).await;
    secrets_round_trip(&make().await).await;
    rewrap_under_the_active_key_is_a_no_op(&make().await).await;
}

/// Checks KEK rotation: `before` seals under the old key; `after` reads the same
/// data with a new active key and the old one retired.
///
/// Only for backends that seal secrets; the in-memory store has nothing to
/// re-wrap.
pub async fn key_rotation(before: &dyn StorageAccessStore, after: &dyn StorageAccessStore) {
    let mut request = credential("cred", "arn:1");
    request.aws_iam_role = AwsIamRoleConfig {
        role_arn: "arn:1".into(),
        secret_access_key: Some("secret".into()),
        ..Default::default()
    }
    .into();
    let expected = request.aws_iam_role.clone();
    before.create_credential(request).await.unwrap();
    before
        .create_external_location(location("loc", "s3://bucket/data", "cred"))
        .await
        .unwrap();

    assert_eq!(after.rewrap_secrets().await.unwrap(), 1);
    assert_eq!(after.rewrap_secrets().await.unwrap(), 0);
    let governed = after
        .governing_location(&storage("s3://bucket/data/x"))
        .await
        .unwrap();
    assert_eq!(governed.credential.aws_iam_role, expected);
}

/// A storage credential named `name`, carrying an IAM role ARN.
pub fn credential(name: &str, role_arn: &str) -> Credential {
    Credential {
        name: name.into(),
        purpose: Purpose::STORAGE.into(),
        comment: Some(format!("{name} comment")),
        aws_iam_role: AwsIamRoleConfig {
            role_arn: role_arn.into(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

/// An external location named `name` at `url`, governed by `credential_name`.
pub fn location(name: &str, url: &str, credential_name: &str) -> ExternalLocation {
    ExternalLocation {
        name: name.into(),
        url: url.into(),
        credential_name: credential_name.into(),
        ..Default::default()
    }
}

fn storage(url: &str) -> StorageLocation {
    url.parse().unwrap()
}

fn role_arn(credential: &Credential) -> Option<&str> {
    credential
        .aws_iam_role
        .as_option()
        .map(|r| r.role_arn.as_str())
}

fn assert_redacted(credential: &Credential) {
    assert!(
        CredentialSecrets::take(&mut credential.clone()).is_empty(),
        "credential '{}' read with secrets",
        credential.name
    );
}

async fn credential_crud(store: &dyn StorageAccessStore) {
    let created = store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    assert_eq!(created.name, "cred");
    assert_eq!(created.full_name, "cred");
    assert_eq!(created.purpose.as_known(), Some(Purpose::STORAGE));
    assert!(created.id.is_some());
    assert!(created.created_at.is_some());
    assert_redacted(&created);

    let got = store.get_credential("cred").await.unwrap();
    assert_eq!(got, created);
    assert_redacted(&got);

    let (listed, next) = store.list_credentials(None, Page::default()).await.unwrap();
    assert_eq!(listed, vec![created.clone()]);
    assert_eq!(next, None);

    let updated = store
        .update_credential(
            "cred",
            CredentialUpdate {
                new_name: Some("renamed".into()),
                comment: Some("new comment".into()),
                read_only: Some(true),
                owner: Some("alice".into()),
                secrets: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.name, "renamed");
    assert_eq!(updated.full_name, "renamed");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.comment.as_deref(), Some("new comment"));
    assert!(updated.read_only);
    assert_eq!(updated.owner.as_deref(), Some("alice"));
    assert_eq!(updated.purpose.as_known(), Some(Purpose::STORAGE));
    assert_redacted(&updated);
    assert!(matches!(
        store.get_credential("cred").await,
        Err(Error::NotFound)
    ));
    assert_eq!(store.get_credential("renamed").await.unwrap(), updated);

    store.delete_credential("renamed").await.unwrap();
    assert!(matches!(
        store.get_credential("renamed").await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.delete_credential("renamed").await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store
            .update_credential("renamed", CredentialUpdate::default())
            .await,
        Err(Error::NotFound)
    ));
}

async fn credential_names_are_case_insensitive(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("Cred", "arn:1"))
        .await
        .unwrap();
    assert!(matches!(
        store.create_credential(credential("cred", "arn:2")).await,
        Err(Error::AlreadyExists)
    ));
    assert_eq!(store.get_credential("CRED").await.unwrap().name, "Cred");

    store
        .create_credential(credential("other", "arn:3"))
        .await
        .unwrap();
    assert!(matches!(
        store
            .update_credential(
                "other",
                CredentialUpdate {
                    new_name: Some("CRED".into()),
                    ..Default::default()
                },
            )
            .await,
        Err(Error::AlreadyExists)
    ));
}

async fn credential_pagination_and_purpose(store: &dyn StorageAccessStore) {
    for name in ["c3", "c1", "c5", "c2", "c4"] {
        store
            .create_credential(credential(name, "arn:1"))
            .await
            .unwrap();
    }
    let mut service = credential("s1", "arn:1");
    service.purpose = Purpose::SERVICE.into();
    store.create_credential(service).await.unwrap();

    let mut names = Vec::new();
    let mut token = None;
    let mut pages = 0;
    loop {
        let (page, next) = store
            .list_credentials(Some(Purpose::STORAGE), Page::new(Some(2), token))
            .await
            .unwrap();
        assert!(page.len() <= 2);
        names.extend(page.into_iter().map(|c| c.name));
        pages += 1;
        match next {
            Some(next) => token = Some(next),
            None => break,
        }
        assert!(pages < 10, "pagination does not terminate");
    }
    assert_eq!(names, ["c1", "c2", "c3", "c4", "c5"]);

    let (all, next) = store.list_credentials(None, Page::default()).await.unwrap();
    assert_eq!(all.len(), 6);
    assert_eq!(next, None);

    let (service, _) = store
        .list_credentials(Some(Purpose::SERVICE), Page::default())
        .await
        .unwrap();
    assert_eq!(service.len(), 1);
    assert_eq!(service[0].name, "s1");
}

async fn external_location_crud(store: &dyn StorageAccessStore) {
    let cred = store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    store
        .create_credential(credential("cred2", "arn:2"))
        .await
        .unwrap();

    let mut request = location("loc", "s3://bucket/data", "cred");
    request.comment = Some("a comment".into());
    request.read_only = true;
    let created = store.create_external_location(request).await.unwrap();
    assert_eq!(created.name, "loc");
    assert_eq!(created.url, "s3://bucket/data");
    assert_eq!(created.credential_name, "cred");
    assert_eq!(Some(&created.credential_id), cred.id.as_ref());
    assert!(created.read_only);
    assert_eq!(created.comment.as_deref(), Some("a comment"));
    assert!(created.external_location_id.is_some());
    assert!(created.created_at.is_some());

    assert_eq!(store.get_external_location("LOC").await.unwrap(), created);
    let (listed, next) = store
        .list_external_locations(Page::default())
        .await
        .unwrap();
    assert_eq!(listed, vec![created.clone()]);
    assert_eq!(next, None);
    assert!(matches!(
        store
            .create_external_location(location("Loc", "s3://other/x", "cred"))
            .await,
        Err(Error::AlreadyExists)
    ));

    let updated = store
        .update_external_location(
            "loc",
            ExternalLocationUpdate {
                new_name: Some("moved".into()),
                url: Some("s3://bucket/elsewhere".into()),
                credential_name: Some("cred2".into()),
                read_only: Some(false),
                comment: Some("moved it".into()),
                owner: Some("bob".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.name, "moved");
    assert_eq!(updated.url, "s3://bucket/elsewhere");
    assert_eq!(updated.credential_name, "cred2");
    assert_ne!(updated.credential_id, created.credential_id);
    assert!(!updated.read_only);
    assert_eq!(updated.comment.as_deref(), Some("moved it"));
    assert_eq!(updated.owner.as_deref(), Some("bob"));
    assert_eq!(updated.external_location_id, created.external_location_id);
    assert!(matches!(
        store.get_external_location("loc").await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store
            .update_external_location(
                "moved",
                ExternalLocationUpdate {
                    credential_name: Some("missing".into()),
                    ..Default::default()
                },
            )
            .await,
        Err(Error::NotFound)
    ));

    store.delete_external_location("moved").await.unwrap();
    assert!(matches!(
        store.get_external_location("moved").await,
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.delete_external_location("moved").await,
        Err(Error::NotFound)
    ));
}

async fn external_location_needs_credential(store: &dyn StorageAccessStore) {
    assert!(matches!(
        store
            .create_external_location(location("loc", "s3://bucket/data", "missing"))
            .await,
        Err(Error::NotFound)
    ));
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    assert!(matches!(
        store
            .create_external_location(location("loc", "https://example.com/x", "cred"))
            .await,
        Err(Error::InvalidArgument(_))
    ));
}

async fn external_locations_never_overlap(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    store
        .create_external_location(location("data", "s3://bucket/data", "cred"))
        .await
        .unwrap();

    for (name, url) in [
        ("nested", "s3://bucket/data/sub"),
        ("parent", "s3://bucket"),
        ("same", "s3://bucket/data/"),
        ("respelled", "https://bucket.s3.amazonaws.com/data"),
    ] {
        assert!(
            matches!(
                store
                    .create_external_location(location(name, url, "cred"))
                    .await,
                Err(Error::InvalidArgument(_))
            ),
            "{url} should overlap s3://bucket/data"
        );
    }

    // Prefixes match on whole segments; other buckets and clouds are unrelated.
    for (name, url) in [
        ("sibling", "s3://bucket/database"),
        ("other_bucket", "s3://bucket2/data"),
        ("other_cloud", "gs://bucket/data"),
    ] {
        store
            .create_external_location(location(name, url, "cred"))
            .await
            .unwrap_or_else(|e| panic!("{url} should not overlap: {e}"));
    }

    // Moving onto another location is refused; keeping its own URL is not.
    assert!(matches!(
        store
            .update_external_location(
                "sibling",
                ExternalLocationUpdate {
                    url: Some("s3://bucket/data/inner".into()),
                    ..Default::default()
                },
            )
            .await,
        Err(Error::InvalidArgument(_))
    ));
    store
        .update_external_location(
            "data",
            ExternalLocationUpdate {
                url: Some("s3://bucket/data".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // A location may move within its own subtree.
    store
        .update_external_location(
            "data",
            ExternalLocationUpdate {
                url: Some("s3://bucket/data/narrower".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
}

async fn credential_rename_follows_to_locations(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    store
        .create_external_location(location("loc", "s3://bucket/data", "cred"))
        .await
        .unwrap();
    store
        .update_credential(
            "cred",
            CredentialUpdate {
                new_name: Some("renamed".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(
        store
            .get_external_location("loc")
            .await
            .unwrap()
            .credential_name,
        "renamed"
    );
}

async fn credential_in_use_cannot_be_deleted(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    store
        .create_external_location(location("loc", "s3://bucket/data", "cred"))
        .await
        .unwrap();
    assert!(matches!(
        store.delete_credential("cred").await,
        Err(Error::InvalidArgument(_))
    ));
    store.delete_external_location("loc").await.unwrap();
    store.delete_credential("cred").await.unwrap();
}

async fn resolves_the_covering_location(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("a", "arn:a"))
        .await
        .unwrap();
    store
        .create_credential(credential("cd", "arn:cd"))
        .await
        .unwrap();
    store
        .create_external_location(location("loc_a", "s3://bucket/a", "a"))
        .await
        .unwrap();
    store
        .create_external_location(location(
            "loc_cd",
            "abfss://ctr@acct.dfs.core.windows.net/c/d",
            "cd",
        ))
        .await
        .unwrap();

    let governed = store
        .governing_location(&storage("s3://bucket/a/x/y.parquet"))
        .await
        .unwrap();
    assert_eq!(governed.location.name, "loc_a");
    assert_eq!(governed.location.credential_name, "a");
    assert_eq!(role_arn(&governed.credential), Some("arn:a"));

    // The root of a location is covered, and any spelling of it resolves.
    for url in [
        "abfss://ctr@acct.dfs.core.windows.net/c/d",
        "https://acct.dfs.core.windows.net/ctr/c/d/e",
    ] {
        let governed = store.governing_location(&storage(url)).await.unwrap();
        assert_eq!(governed.location.name, "loc_cd", "{url}");
        assert_eq!(role_arn(&governed.credential), Some("arn:cd"));
    }

    for url in [
        "s3://bucket",
        "s3://bucket/ab",
        "abfss://ctr@acct.dfs.core.windows.net/c",
        "gs://bucket/a/x",
    ] {
        assert!(
            matches!(
                store.governing_location(&storage(url)).await,
                Err(Error::NotFound)
            ),
            "{url} is not covered"
        );
        assert_eq!(store.covering_location(&storage(url)).await.unwrap(), None);
    }

    let covering = store
        .covering_location(&storage("s3://bucket/a/x"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(covering, governed.location);

    // Moving a location moves what it governs.
    store
        .update_external_location(
            "loc_a",
            ExternalLocationUpdate {
                url: Some("s3://bucket/b".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(matches!(
        store.governing_location(&storage("s3://bucket/a/x")).await,
        Err(Error::NotFound)
    ));
    assert_eq!(
        store
            .governing_location(&storage("s3://bucket/b/x"))
            .await
            .unwrap()
            .location
            .name,
        "loc_a"
    );
}

async fn secrets_round_trip(store: &dyn StorageAccessStore) {
    let mut request = credential("cred", "arn:1");
    request.aws_iam_role = AwsIamRoleConfig {
        role_arn: "arn:1".into(),
        region: Some("eu-west-1".into()),
        access_key_id: Some("AKIA".into()),
        secret_access_key: Some("secret".into()),
        ..Default::default()
    }
    .into();
    let expected = request.aws_iam_role.clone();
    store.create_credential(request).await.unwrap();
    store
        .create_external_location(location("loc", "s3://bucket/data", "cred"))
        .await
        .unwrap();

    let governed = store
        .governing_location(&storage("s3://bucket/data/x"))
        .await
        .unwrap();
    assert_eq!(governed.credential.aws_iam_role, expected);

    // Replacing the secrets replaces the whole configuration.
    let replacement = CredentialSecrets::take(&mut Credential {
        azure_storage_key: AzureStorageKey {
            account_name: "acct".into(),
            account_key: "key".into(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    });
    let updated = store
        .update_credential(
            "cred",
            CredentialUpdate {
                secrets: Some(replacement),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_redacted(&updated);
    let governed = store
        .governing_location(&storage("s3://bucket/data/x"))
        .await
        .unwrap();
    assert!(governed.credential.aws_iam_role.is_unset());
    assert_eq!(
        governed
            .credential
            .azure_storage_key
            .as_option()
            .map(|k| k.account_key.as_str()),
        Some("key")
    );
    assert_redacted(&store.get_credential("cred").await.unwrap());
}

async fn rewrap_under_the_active_key_is_a_no_op(store: &dyn StorageAccessStore) {
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    assert_eq!(store.rewrap_secrets().await.unwrap(), 0);
    assert_eq!(
        role_arn(&governed_credential(store, "cred").await),
        Some("arn:1")
    );
}

async fn governed_credential(store: &dyn StorageAccessStore, name: &str) -> Credential {
    store
        .create_external_location(location("rewrap", "s3://rewrap/", name))
        .await
        .unwrap();
    store
        .governing_location(&storage("s3://rewrap/x"))
        .await
        .unwrap()
        .credential
}
