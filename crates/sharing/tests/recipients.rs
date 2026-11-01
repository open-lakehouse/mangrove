//! The token rules over the in-memory store.

use olai_store::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_sharing::recipients::{
    create_recipient, pending_activation, redeem, rewrap_tokens, rotate_token,
};
use unitycatalog_sharing::tokens::token_hash;
use unitycatalog_sharing::{Error, InMemorySharingStore, RecipientStore, TokenReader};

fn encryptor(active: &str, keys: &[&str]) -> EnvelopeEncryptor {
    let keys = keys
        .iter()
        .map(|k| ((*k).into(), vec![k.as_bytes()[0]; 32]));
    EnvelopeEncryptor::local(LocalKeyProvider::new(active, keys).unwrap())
}

#[tokio::test]
async fn an_activation_code_hands_out_the_token_once() {
    let store = InMemorySharingStore::new();
    let keys = encryptor("a", &["a"]);
    let recipient = create_recipient(&store, &keys, "acme", None, Some(i64::MAX))
        .await
        .unwrap();
    let token = store.tokens_for_recipient(&recipient.id).await.unwrap()[0].clone();
    let code = pending_activation(&keys, &token).await.unwrap().unwrap();

    let activated = redeem(&store, &keys, &code).await.unwrap();
    assert_eq!(activated.token.id, token.id);
    let (_, found) = store
        .token_by_hash(&token_hash(&activated.bearer_token))
        .await
        .unwrap();
    assert_eq!(found, recipient);

    assert!(matches!(
        redeem(&store, &keys, &code).await,
        Err(Error::NotFound)
    ));
    let token = store.tokens_for_recipient(&recipient.id).await.unwrap()[0].clone();
    assert_eq!(pending_activation(&keys, &token).await.unwrap(), None);
}

#[tokio::test]
async fn rotation_keeps_the_expiry_and_a_grace_period() {
    let store = InMemorySharingStore::new();
    let keys = encryptor("a", &["a"]);
    let recipient = create_recipient(&store, &keys, "acme", None, Some(i64::MAX))
        .await
        .unwrap();
    assert!(matches!(
        rotate_token(&store, &keys, &recipient, Some(-1)).await,
        Err(Error::InvalidArgument(_))
    ));

    rotate_token(&store, &keys, &recipient, Some(60))
        .await
        .unwrap();
    let tokens = store.tokens_for_recipient(&recipient.id).await.unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].expires_at, Some(i64::MAX));
    assert!(tokens[1].expires_at.unwrap() <= tokens[0].created_at + 60_000);

    rotate_token(&store, &keys, &recipient, None).await.unwrap();
    assert_eq!(
        store
            .tokens_for_recipient(&recipient.id)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn sealed_tokens_move_to_the_active_key() {
    let store = InMemorySharingStore::new();
    let before = encryptor("a", &["a"]);
    let recipient = create_recipient(&store, &before, "acme", None, None)
        .await
        .unwrap();

    let after = encryptor("b", &["a", "b"]);
    assert_eq!(rewrap_tokens(&store, &after).await.unwrap(), 1);
    assert_eq!(rewrap_tokens(&store, &after).await.unwrap(), 0);

    // The retired key is no longer needed to redeem the code.
    let only_b = encryptor("b", &["b"]);
    let token = store.tokens_for_recipient(&recipient.id).await.unwrap()[0].clone();
    let code = pending_activation(&only_b, &token).await.unwrap().unwrap();
    redeem(&store, &only_b, &code).await.unwrap();
}
