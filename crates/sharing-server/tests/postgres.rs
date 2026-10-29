//! The Postgres store against a live database.
//!
//! Runs only when `SHARING_SERVER_TEST_DATABASE_URL` holds the connection URL
//! of an empty, disposable Postgres database (it is migrated and written to) —
//! for example a throwaway `postgres:17` container — and is skipped otherwise:
//!
//! ```sh
//! SHARING_SERVER_TEST_DATABASE_URL=<connection url> \
//!   cargo test -p olai-sharing-server --test postgres
//! ```
#![cfg(feature = "postgres")]

use open_sharing_server::Error;
use open_sharing_server::store::postgres::{
    PostgresStore, connect, ensure_schema_current, migrate,
};
use open_sharing_server::store::{
    ExternalLocationRecord, SealedCredential, ShareObject, ShareObjectKind, ShareStore, TokenRecord,
};

#[tokio::test]
async fn postgres_store_round_trips() {
    let Ok(url) = std::env::var("SHARING_SERVER_TEST_DATABASE_URL") else {
        eprintln!("SHARING_SERVER_TEST_DATABASE_URL not set; skipping");
        return;
    };
    let pool = connect(&url, 2).await.unwrap();
    assert!(
        ensure_schema_current(&pool).await.is_err(),
        "pending migrations must be reported"
    );
    migrate(&pool).await.unwrap();
    ensure_schema_current(&pool).await.unwrap();
    let store = PostgresStore::new(pool);

    // Shares and their objects.
    let share = store
        .create_share("pg_share", Some("c".into()))
        .await
        .unwrap();
    assert!(matches!(
        store.create_share("pg_share", None).await,
        Err(Error::InvalidArgument(_))
    ));
    let object = ShareObject {
        start_version: Some(3),
        ..ShareObject::new(ShareObjectKind::Table, "main.sales.orders")
    };
    let updated = store
        .update_share("pg_share", vec![object.clone()], None)
        .await
        .unwrap();
    assert_eq!(updated.id, share.id);
    assert_eq!(
        store.get_share("pg_share").await.unwrap().objects,
        vec![object]
    );

    // Recipients, tokens and grants.
    let recipient = store.create_recipient("pg_recipient", None).await.unwrap();
    store
        .insert_token(TokenRecord {
            id: "t1".into(),
            recipient_id: recipient.id.clone(),
            token_hash: vec![1, 2, 3],
            sealed_token: Some(vec![9]),
            activation_code: Some("code".into()),
            expires_at: None,
            created_at: 1,
        })
        .await
        .unwrap();
    let (token, found) = store.token_by_hash(&[1, 2, 3]).await.unwrap();
    assert_eq!(found.name, "pg_recipient");
    assert_eq!(
        store.token_by_activation_code("code").await.unwrap().id,
        token.id
    );
    assert_eq!(store.sealed_tokens().await.unwrap().len(), 1);
    store.mark_activated("t1").await.unwrap();
    assert!(matches!(
        store.token_by_activation_code("code").await,
        Err(Error::NotFound)
    ));
    assert!(store.sealed_tokens().await.unwrap().is_empty());

    assert!(!store.is_granted("pg_share", &recipient.id).await.unwrap());
    store.grant("pg_share", "pg_recipient").await.unwrap();
    store.grant("pg_share", "pg_recipient").await.unwrap();
    assert!(store.is_granted("pg_share", &recipient.id).await.unwrap());
    assert_eq!(
        store.recipients_of("pg_share").await.unwrap(),
        ["pg_recipient"]
    );
    assert_eq!(
        store.shares_of(&recipient.id).await.unwrap()[0].name,
        "pg_share"
    );
    assert!(matches!(
        store.grant("pg_share", "nobody").await,
        Err(Error::NotFound)
    ));
    store.revoke("pg_share", "pg_recipient").await.unwrap();
    assert!(!store.is_granted("pg_share", &recipient.id).await.unwrap());

    // Sealed credentials and external locations.
    store
        .put_credential(SealedCredential {
            name: "cred".into(),
            sealed: vec![7],
        })
        .await
        .unwrap();
    store
        .put_external_location(ExternalLocationRecord {
            name: "loc".into(),
            url: "s3://bucket/prefix".into(),
            credential_name: "cred".into(),
        })
        .await
        .unwrap();
    assert_eq!(store.list_external_locations().await.unwrap().len(), 1);
    assert_eq!(store.get_credential("cred").await.unwrap().sealed, vec![7]);

    // Deleting a recipient cascades to its tokens and grants.
    store.grant("pg_share", "pg_recipient").await.unwrap();
    store.delete_recipient("pg_recipient").await.unwrap();
    assert!(matches!(
        store.token_by_hash(&[1, 2, 3]).await,
        Err(Error::NotFound)
    ));
    assert!(store.recipients_of("pg_share").await.unwrap().is_empty());
    store.delete_share("pg_share").await.unwrap();
    assert!(matches!(
        store.get_share("pg_share").await,
        Err(Error::NotFound)
    ));
}
