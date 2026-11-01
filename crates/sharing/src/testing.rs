//! The behavior every [`SharingStore`] shares (feature `testing`).
//!
//! Every store implementation runs [`round_trips`] against an empty store in
//! its tests; it panics on the first divergence. It covers the atomicity the
//! [`RecipientStore`](crate::RecipientStore) contract promises, among
//! them that of concurrent redemptions of one activation code exactly one
//! succeeds.

use crate::{
    Error, RecipientRecord, RecipientUpdate, ShareObject, ShareObjectKind, SharingStore,
    TokenRecord,
};

fn recipient_record(id: &str, name: &str) -> RecipientRecord {
    RecipientRecord {
        id: id.into(),
        name: name.into(),
        comment: None,
        created_at: 1,
    }
}

/// A pending token: its digest is its id, its activation code `code-{id}`.
fn token_record(id: &str, recipient_id: &str, created_at: i64) -> TokenRecord {
    TokenRecord {
        id: id.into(),
        recipient_id: recipient_id.into(),
        token_hash: id.as_bytes().to_vec(),
        sealed_token: Some(vec![9]),
        activation_code_hash: Some(code(id)),
        expires_at: None,
        created_at,
    }
}

fn code(token_id: &str) -> Vec<u8> {
    format!("code-{token_id}").into_bytes()
}

async fn token_ids(store: &dyn SharingStore, recipient_id: &str) -> Vec<String> {
    store
        .tokens_for_recipient(recipient_id)
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.id)
        .collect()
}

/// Exercise every store operation against an empty `store`.
pub async fn round_trips(store: &dyn SharingStore) {
    // Shares and their objects.
    let share = store
        .create_share("conformance_share", Some("c".into()))
        .await
        .unwrap();
    assert!(matches!(
        store.create_share("conformance_share", None).await,
        Err(Error::AlreadyExists)
    ));
    let object = ShareObject {
        start_version: Some(3),
        ..ShareObject::new(ShareObjectKind::Table, "main.sales.orders")
    };
    let updated = store
        .update_share("conformance_share", vec![object.clone()], None)
        .await
        .unwrap();
    assert_eq!(updated.id, share.id);
    assert_eq!(
        store.get_share("conformance_share").await.unwrap().objects,
        vec![object]
    );

    // Recipients and tokens. Creating one with a taken name leaves no token.
    let recipient = recipient_record("r1", "conformance_recipient");
    store
        .create_recipient(recipient.clone(), token_record("t1", "r1", 1))
        .await
        .unwrap();
    assert!(matches!(
        store
            .create_recipient(
                recipient_record("r2", "conformance_recipient"),
                token_record("t-dup", "r2", 1),
            )
            .await,
        Err(Error::AlreadyExists)
    ));
    assert!(matches!(
        store.token_by_hash(b"t-dup").await,
        Err(Error::NotFound)
    ));
    let (token, found) = store.token_by_hash(b"t1").await.unwrap();
    assert_eq!((token.id.as_str(), found), ("t1", recipient.clone()));

    // Updates: the comment, and the newest token's expiry.
    let updated = store
        .update_recipient(
            "conformance_recipient",
            RecipientUpdate {
                comment: Some("updated".into()),
                expires_at: Some(Some(1_000)),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.comment.as_deref(), Some("updated"));
    assert_eq!(updated.id, recipient.id);
    let kept = store
        .update_recipient("conformance_recipient", RecipientUpdate::default())
        .await
        .unwrap();
    assert_eq!(kept.comment.as_deref(), Some("updated"));
    let tokens = store.tokens_for_recipient("r1").await.unwrap();
    assert_eq!(tokens[0].expires_at, Some(1_000));

    // An activation code is good once, however many redeem it at once, and
    // only while its token is live.
    assert!(matches!(
        store.redeem_activation(&code("t1"), 1_000).await,
        Err(Error::NotFound)
    ));
    let t1_code = code("t1");
    let (first, second) = tokio::join!(
        store.redeem_activation(&t1_code, 2),
        store.redeem_activation(&t1_code, 2),
    );
    let redeemed = match (first, second) {
        (Ok(t), Err(Error::NotFound)) | (Err(Error::NotFound), Ok(t)) => t,
        other => panic!("exactly one redemption should succeed: {other:?}"),
    };
    assert_eq!(redeemed.sealed_token, Some(vec![9]));
    assert!(matches!(
        store.redeem_activation(&code("t1"), 2).await,
        Err(Error::NotFound)
    ));
    let (activated, _) = store.token_by_hash(b"t1").await.unwrap();
    assert_eq!(
        (activated.sealed_token, activated.activation_code_hash),
        (None, None)
    );

    // Rotation: the other tokens keep a grace period, never extended, and are
    // dropped once expired; without one, they go at once.
    store
        .rotate_token(token_record("t2", "r1", 10), Some(100))
        .await
        .unwrap();
    assert_eq!(token_ids(store, "r1").await, ["t2", "t1"]);
    let tokens = store.tokens_for_recipient("r1").await.unwrap();
    assert_eq!(tokens[1].expires_at, Some(100));
    store
        .rotate_token(token_record("t3", "r1", 200), Some(5_000))
        .await
        .unwrap();
    assert_eq!(token_ids(store, "r1").await, ["t3", "t2"]);
    store
        .rotate_token(token_record("t4", "r1", 300), None)
        .await
        .unwrap();
    assert_eq!(token_ids(store, "r1").await, ["t4"]);
    // Tokens created in the same millisecond still list newest first.
    store
        .rotate_token(token_record("t5", "r1", 300), Some(400))
        .await
        .unwrap();
    assert_eq!(token_ids(store, "r1").await, ["t5", "t4"]);

    // Re-sealing replaces only the copy it read, and never revives one.
    assert_eq!(store.sealed_tokens().await.unwrap().len(), 2);
    assert!(store.reseal_token("t5", &[9], vec![8]).await.unwrap());
    assert!(!store.reseal_token("t5", &[9], vec![7]).await.unwrap());
    store.redeem_activation(&code("t5"), 300).await.unwrap();
    assert!(!store.reseal_token("t5", &[8], vec![6]).await.unwrap());
    let (t5, _) = store.token_by_hash(b"t5").await.unwrap();
    assert_eq!(t5.sealed_token, None);

    // Grants.
    assert!(
        !store
            .is_granted("conformance_share", &recipient.id)
            .await
            .unwrap()
    );
    store
        .grant("conformance_share", "conformance_recipient")
        .await
        .unwrap();
    store
        .grant("conformance_share", "conformance_recipient")
        .await
        .unwrap();
    assert!(
        store
            .is_granted("conformance_share", &recipient.id)
            .await
            .unwrap()
    );
    assert_eq!(
        store.recipients_of("conformance_share").await.unwrap(),
        ["conformance_recipient"]
    );
    assert_eq!(
        store.shares_of(&recipient.id).await.unwrap()[0].name,
        "conformance_share"
    );
    assert!(matches!(
        store.grant("conformance_share", "nobody").await,
        Err(Error::NotFound)
    ));
    store
        .revoke("conformance_share", "conformance_recipient")
        .await
        .unwrap();
    assert!(
        !store
            .is_granted("conformance_share", &recipient.id)
            .await
            .unwrap()
    );

    // Deleting a recipient cascades to its tokens and grants.
    store
        .grant("conformance_share", "conformance_recipient")
        .await
        .unwrap();
    store
        .delete_recipient("conformance_recipient")
        .await
        .unwrap();
    assert!(matches!(
        store.token_by_hash(b"t5").await,
        Err(Error::NotFound)
    ));
    assert!(store.tokens_for_recipient("r1").await.unwrap().is_empty());
    assert!(
        store
            .recipients_of("conformance_share")
            .await
            .unwrap()
            .is_empty()
    );
    store.delete_share("conformance_share").await.unwrap();
    assert!(matches!(
        store.get_share("conformance_share").await,
        Err(Error::NotFound)
    ));
}
