//! The Postgres directory ([`NeonUsers`], [`PgAuthz`]) against Neon Auth's
//! `user` table layout and olai-auth's `authz` migrations, and the grants
//! API's principal resolution over them.
//!
//! Gated behind `integration-pg`; `#[sqlx::test]` provisions a fresh database.
//! Run with `DATABASE_URL` pointing at a live Postgres:
//!
//! ```sh
//! cargo test -p olai-uc-server --features integration-pg --test directory_pg
//! ```
#![cfg(feature = "integration-pg")]

use std::sync::Arc;

use olai_auth::grants::{Subject, SubjectKind};
use sqlx::PgPool;
use unitycatalog_server::directory::postgres::{NeonUsers, PgAuthz};
use unitycatalog_server::directory::{AuthzDirectory, Directory, Member, UserSource};
use unitycatalog_server::grants::PrincipalDirectory;

const ALICE: &str = "00000000-0000-0000-0000-000000000001";
const BOB: &str = "00000000-0000-0000-0000-000000000002";
const CAROL: &str = "00000000-0000-0000-0000-000000000003";

async fn setup(pool: &PgPool) -> (String, String) {
    for statement in [
        r#"CREATE SCHEMA neon_auth"#.to_string(),
        r#"CREATE TABLE neon_auth."user" (id uuid PRIMARY KEY, name text NOT NULL,
           email text NOT NULL, role text, banned bool, "banExpires" timestamptz)"#
            .to_string(),
        format!(
            r#"INSERT INTO neon_auth."user" VALUES
               ('{ALICE}', 'Alice Liddell', 'alice@example.com', NULL, NULL, NULL),
               ('{BOB}', '', 'Bob@Example.com', NULL, true, now() - interval '1 day'),
               ('{CAROL}', 'Carol', 'carol@example.com', NULL, true, NULL)"#
        ),
    ] {
        sqlx::query(&statement).execute(pool).await.unwrap();
    }
    olai_auth::migrate(pool, "authz").await.unwrap();
    let group: String = sqlx::query_scalar(
        "INSERT INTO authz.groups (name) VALUES ('analysts') RETURNING id::text",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let account: String = sqlx::query_scalar(
        "INSERT INTO authz.service_accounts (name, description) VALUES ('etl', 'nightly') RETURNING id::text",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO authz.service_accounts (name, disabled_at) VALUES ('old', now())")
        .execute(pool)
        .await
        .unwrap();
    for (kind, id) in [("user", ALICE), ("service_account", account.as_str())] {
        sqlx::query(
            "INSERT INTO authz.memberships (group_id, principal_type, principal_id) VALUES ($1::uuid, $2, $3)",
        )
        .bind(&group)
        .bind(kind)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }
    (group, account)
}

#[sqlx::test(migrations = false)]
async fn neon_users(pool: PgPool) {
    setup(&pool).await;
    let users = NeonUsers::new(pool, "neon_auth");

    let all = users.users().await.unwrap();
    let names: Vec<_> = all.iter().map(|u| u.user_name.as_str()).collect();
    assert_eq!(
        names,
        ["alice@example.com", "Bob@Example.com", "carol@example.com"]
    );
    // An empty name is no display name; a lapsed ban is active, a standing one not.
    assert_eq!(all[0].display_name.as_deref(), Some("Alice Liddell"));
    assert_eq!(all[1].display_name, None);
    assert!(all[1].active);
    assert!(!all[2].active);

    assert_eq!(
        users.user(ALICE).await.unwrap().unwrap().emails,
        ["alice@example.com"]
    );
    assert_eq!(
        users
            .user_by_name("bob@example.com")
            .await
            .unwrap()
            .unwrap()
            .id,
        BOB
    );
    assert!(users.user("nobody").await.unwrap().is_none());
}

#[sqlx::test(migrations = false)]
async fn authz_groups_and_accounts(pool: PgPool) {
    let (group, account) = setup(&pool).await;
    let authz = PgAuthz::new(pool, "authz");

    let groups = authz.groups().await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(
        groups[0].members,
        [
            Member {
                kind: SubjectKind::User,
                id: ALICE.into()
            },
            Member {
                kind: SubjectKind::ServiceAccount,
                id: account.clone()
            },
        ]
    );
    assert_eq!(authz.group(&group).await.unwrap().unwrap().name, "analysts");
    assert_eq!(
        authz.group_by_name("analysts").await.unwrap().unwrap().id,
        group
    );

    let accounts = authz.service_principals().await.unwrap();
    assert_eq!(
        accounts
            .iter()
            .map(|a| (a.name.as_str(), a.active))
            .collect::<Vec<_>>(),
        [("etl", true), ("old", false)]
    );
    assert_eq!(
        authz
            .service_principal(&account)
            .await
            .unwrap()
            .unwrap()
            .description
            .as_deref(),
        Some("nightly")
    );
    assert!(
        authz
            .service_principal_by_name("nope")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrations = false)]
async fn grants_principals_resolve_through_the_directory(pool: PgPool) {
    let (group, account) = setup(&pool).await;
    let directory = Directory::new(
        Arc::new(NeonUsers::new(pool.clone(), "neon_auth")),
        Arc::new(PgAuthz::new(pool, "authz")),
    );
    assert_eq!(
        directory.resolve("ALICE@example.com").await.unwrap(),
        Subject::user(ALICE)
    );
    assert_eq!(
        directory.resolve("analysts").await.unwrap(),
        Subject::group(group.clone())
    );
    assert_eq!(
        directory.resolve("etl").await.unwrap(),
        Subject::service_account(account)
    );
    assert!(directory.resolve("nobody@example.com").await.is_err());
    assert_eq!(
        directory.name(&Subject::user(ALICE)).await.unwrap(),
        "alice@example.com"
    );
    assert_eq!(
        directory.name(&Subject::group(group)).await.unwrap(),
        "analysts"
    );
}
