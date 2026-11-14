//! [`PgNeonUserStatus`] against Neon Auth's `user` table layout.
//!
//! Gated behind `integration-pg`; `#[sqlx::test]` provisions a fresh database.
//! Run with `DATABASE_URL` pointing at a live Postgres:
//!
//! ```sh
//! cargo test -p olai-uc-server --features integration-pg --test neon_user_status
//! ```
#![cfg(feature = "integration-pg")]

use olai_auth::{UserStatus, UserStatusStore};
use sqlx::PgPool;
use unitycatalog_server::rest::PgNeonUserStatus;

const ISSUER: &str = "https://auth.example.com/neondb/auth";

/// The columns of Neon Auth's `neon_auth."user"` the store reads, plus one it
/// must not need.
async fn neon_auth(pool: &PgPool, schema: &str) {
    for statement in [
        format!(r#"CREATE SCHEMA "{schema}""#),
        format!(
            r#"CREATE TABLE "{schema}"."user" (id uuid PRIMARY KEY, email text, role text,
               banned bool, "banExpires" timestamptz)"#
        ),
        format!(
            r#"INSERT INTO "{schema}"."user" VALUES
               ('00000000-0000-0000-0000-000000000001', 'admin@example.com', 'admin', false, NULL),
               ('00000000-0000-0000-0000-000000000002', 'user@example.com', NULL, NULL, NULL),
               ('00000000-0000-0000-0000-000000000003', 'multi@example.com', 'user, admin', NULL, NULL),
               ('00000000-0000-0000-0000-000000000004', 'banned@example.com', 'user', true, NULL),
               ('00000000-0000-0000-0000-000000000005', 'lapsed@example.com', 'user', true, now() - interval '1 day'),
               ('00000000-0000-0000-0000-000000000006', 'until@example.com', 'user', true, now() + interval '1 day')"#
        ),
    ] {
        sqlx::query(&statement).execute(pool).await.unwrap();
    }
}

async fn status(store: &PgNeonUserStatus, id: u8) -> Option<UserStatus> {
    store
        .status(ISSUER, &format!("00000000-0000-0000-0000-00000000000{id}"))
        .await
        .unwrap()
}

#[sqlx::test(migrations = false)]
async fn reads_role_and_ban(pool: PgPool) {
    neon_auth(&pool, "neon_auth").await;
    let store = PgNeonUserStatus::new(pool, "neon_auth");

    let admin = UserStatus {
        banned: false,
        admin: true,
    };
    let user = UserStatus {
        banned: false,
        admin: false,
    };
    let banned = UserStatus {
        banned: true,
        admin: false,
    };
    assert_eq!(status(&store, 1).await, Some(admin));
    assert_eq!(status(&store, 2).await, Some(user));
    assert_eq!(status(&store, 3).await, Some(admin));
    assert_eq!(status(&store, 4).await, Some(banned));
    // A ban that has expired no longer counts; one that hasn't yet does.
    assert_eq!(status(&store, 5).await, Some(user));
    assert_eq!(status(&store, 6).await, Some(banned));
    // Unknown users, and subjects that aren't user ids at all.
    assert_eq!(status(&store, 9).await, None);
    assert_eq!(store.status(ISSUER, "not-a-uuid").await.unwrap(), None);
}

#[sqlx::test(migrations = false)]
async fn reads_another_schema(pool: PgPool) {
    neon_auth(&pool, "auth").await;
    let store = PgNeonUserStatus::new(pool.clone(), "auth");
    assert!(status(&store, 1).await.is_some());

    // The default schema isn't there, which is an error rather than "unknown".
    let missing = PgNeonUserStatus::new(pool, "neon_auth");
    assert!(missing.status(ISSUER, "x").await.is_err());
}
