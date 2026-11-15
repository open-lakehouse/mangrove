//! A read-only SCIM 2.0 API over the [`Directory`], shaped like Databricks'
//! (`/api/2.0/preview/scim/v2`): `Me`, `Users`, `Groups` and
//! `ServicePrincipals`, with RFC 7644 filters (`filter=userName co "ali"`) and
//! paging (`startIndex`, 1-based, and `count`).
//!
//! Every endpoint needs a signed-in caller, unless the server runs without
//! authentication (`auth.mode: anonymous`), where anyone may read.
//! `Me` answers the caller as a user (or a service principal), from the
//! directory where it knows the caller, else from the caller's name alone.
//! Attribute selection (`attributes`, `excludedAttributes`) and sorting are not
//! supported; resources are ordered as their source lists them.

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use olai_auth::grants::{Subject, SubjectKind};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::RequestContext;
use crate::directory::{Directory, Group, Member, ServicePrincipal, User};
use crate::policy::Principal;

pub mod filter;

/// Where the API is mounted.
pub const SCIM_BASE_PATH: &str = "/api/2.0/preview/scim/v2";

const USER_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
const GROUP_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";
const SERVICE_PRINCIPAL_SCHEMA: &str = "urn:ietf:params:scim:schemas:core:2.0:ServicePrincipal";
const LIST_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
const ERROR_SCHEMA: &str = "urn:ietf:params:scim:api:messages:2.0:Error";

/// The page size when `count` is unset.
pub const DEFAULT_COUNT: usize = 100;
/// The largest page served.
pub const MAX_COUNT: usize = 1000;

/// What the SCIM handlers read.
#[derive(Clone, Debug)]
pub struct ScimState {
    directory: Directory,
    allow_anonymous: bool,
}

impl ScimState {
    /// The SCIM API over `directory`. With `allow_anonymous`, callers need not
    /// be signed in (for a server without authentication).
    pub fn new(directory: Directory, allow_anonymous: bool) -> Self {
        Self {
            directory,
            allow_anonymous,
        }
    }
}

/// The SCIM routes, under [`SCIM_BASE_PATH`].
pub fn router(state: ScimState) -> Router {
    let routes = Router::new()
        .route("/Me", get(me))
        .route("/Users", get(list_users))
        .route("/Users/{id}", get(get_user))
        .route("/Groups", get(list_groups))
        .route("/Groups/{id}", get(get_group))
        .route("/ServicePrincipals", get(list_service_principals))
        .route("/ServicePrincipals/{id}", get(get_service_principal))
        .with_state(state);
    Router::new().nest(SCIM_BASE_PATH, routes)
}

/// A SCIM error response (RFC 7644 §3.12).
#[derive(Debug)]
pub struct ScimError {
    status: StatusCode,
    detail: String,
    scim_type: Option<&'static str>,
}

impl ScimError {
    fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
            scim_type: None,
        }
    }

    fn invalid_filter(detail: String) -> Self {
        Self {
            scim_type: Some("invalidFilter"),
            ..Self::new(StatusCode::BAD_REQUEST, detail)
        }
    }

    fn not_found(what: &str, id: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, format!("{what} {id} not found"))
    }
}

impl From<crate::Error> for ScimError {
    fn from(e: crate::Error) -> Self {
        tracing::error!(error = %e, "scim directory lookup failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the directory could not be read",
        )
    }
}

impl IntoResponse for ScimError {
    fn into_response(self) -> Response {
        let mut body = json!({
            "schemas": [ERROR_SCHEMA],
            "detail": self.detail,
            "status": self.status.as_u16().to_string(),
        });
        if let Some(scim_type) = self.scim_type {
            body["scimType"] = json!(scim_type);
        }
        scim_json(self.status, body)
    }
}

fn scim_json(status: StatusCode, body: Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/scim+json")],
        Json(body),
    )
        .into_response()
}

type ScimResult = Result<Response, ScimError>;

/// The list parameters (RFC 7644 §3.4.2).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    filter: Option<String>,
    start_index: Option<usize>,
    count: Option<usize>,
}

fn signed_in(cx: &RequestContext) -> bool {
    #[cfg(feature = "olai-auth")]
    if cx.verified_principal().is_some() {
        return true;
    }
    matches!(cx.recipient(), Principal::User(_))
}

fn require_caller(state: &ScimState, cx: &RequestContext) -> Result<(), ScimError> {
    if state.allow_anonymous || signed_in(cx) {
        Ok(())
    } else {
        Err(ScimError::new(StatusCode::UNAUTHORIZED, "sign in first"))
    }
}

/// `resources` filtered by `params.filter` and paged, as a `ListResponse`.
fn list_response(resources: Vec<Value>, params: &ListParams) -> ScimResult {
    let resources = match params.filter.as_deref().filter(|f| !f.trim().is_empty()) {
        Some(input) => {
            let filter = filter::parse(input).map_err(ScimError::invalid_filter)?;
            resources
                .into_iter()
                .filter(|r| filter::matches(&filter, r))
                .collect()
        }
        None => resources,
    };
    let total = resources.len();
    let start = params.start_index.unwrap_or(1).max(1);
    let count = params.count.unwrap_or(DEFAULT_COUNT).min(MAX_COUNT);
    let page: Vec<Value> = resources.into_iter().skip(start - 1).take(count).collect();
    Ok(scim_json(
        StatusCode::OK,
        json!({
            "schemas": [LIST_SCHEMA],
            "totalResults": total,
            "startIndex": start,
            "itemsPerPage": page.len(),
            "Resources": page,
        }),
    ))
}

fn ref_path(kind: SubjectKind, id: &str) -> String {
    match kind {
        SubjectKind::User => format!("Users/{id}"),
        SubjectKind::Group => format!("Groups/{id}"),
        SubjectKind::ServiceAccount => format!("ServicePrincipals/{id}"),
    }
}

fn group_refs(groups: &[&Group]) -> Vec<Value> {
    groups
        .iter()
        .map(|g| json!({ "value": g.id, "display": g.name, "$ref": ref_path(SubjectKind::Group, &g.id) }))
        .collect()
}

fn user_resource(user: &User, groups: &[&Group]) -> Value {
    let mut resource = json!({
        "schemas": [USER_SCHEMA],
        "id": user.id,
        "userName": user.user_name,
        "active": user.active,
        "emails": user
            .emails
            .iter()
            .enumerate()
            .map(|(i, e)| json!({ "value": e, "type": "work", "primary": i == 0 }))
            .collect::<Vec<_>>(),
        "groups": group_refs(groups),
    });
    if let Some(name) = &user.display_name {
        resource["displayName"] = json!(name);
    }
    resource
}

fn service_principal_resource(account: &ServicePrincipal, groups: &[&Group]) -> Value {
    json!({
        "schemas": [SERVICE_PRINCIPAL_SCHEMA],
        "id": account.id,
        "applicationId": account.id,
        "displayName": account.name,
        "active": account.active,
        "groups": group_refs(groups),
    })
}

/// The groups `member` is directly in.
fn groups_of<'a>(groups: &'a [Group], kind: SubjectKind, id: &str) -> Vec<&'a Group> {
    let member = Member {
        kind,
        id: id.to_string(),
    };
    groups
        .iter()
        .filter(|g| g.members.contains(&member))
        .collect()
}

/// Group resources, with each member's display name looked up once.
async fn group_resources(directory: &Directory, groups: &[Group]) -> Result<Vec<Value>, ScimError> {
    let mut names: HashMap<Member, Option<String>> = HashMap::new();
    let mut resources = Vec::with_capacity(groups.len());
    for group in groups {
        let mut members = Vec::with_capacity(group.members.len());
        for member in &group.members {
            if !names.contains_key(member) {
                let subject = Subject {
                    kind: member.kind,
                    id: member.id.clone(),
                };
                let name = directory.subject_name(&subject).await?;
                names.insert(member.clone(), name);
            }
            let display = names[member].clone().unwrap_or_else(|| member.id.clone());
            members.push(json!({
                "value": member.id,
                "display": display,
                "$ref": ref_path(member.kind, &member.id),
            }));
        }
        resources.push(json!({
            "schemas": [GROUP_SCHEMA],
            "id": group.id,
            "displayName": group.name,
            "members": members,
        }));
    }
    Ok(resources)
}

async fn list_users(
    State(state): State<ScimState>,
    cx: RequestContext,
    Query(params): Query<ListParams>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let users = state.directory.users().users().await?;
    let groups = state.directory.authz().groups().await?;
    let resources = users
        .iter()
        .map(|u| user_resource(u, &groups_of(&groups, SubjectKind::User, &u.id)))
        .collect();
    list_response(resources, &params)
}

async fn get_user(
    State(state): State<ScimState>,
    cx: RequestContext,
    Path(id): Path<String>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let user = state
        .directory
        .users()
        .user(&id)
        .await?
        .ok_or_else(|| ScimError::not_found("User", &id))?;
    let groups = state.directory.authz().groups().await?;
    Ok(scim_json(
        StatusCode::OK,
        user_resource(&user, &groups_of(&groups, SubjectKind::User, &user.id)),
    ))
}

async fn list_groups(
    State(state): State<ScimState>,
    cx: RequestContext,
    Query(params): Query<ListParams>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let groups = state.directory.authz().groups().await?;
    let resources = group_resources(&state.directory, &groups).await?;
    list_response(resources, &params)
}

async fn get_group(
    State(state): State<ScimState>,
    cx: RequestContext,
    Path(id): Path<String>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let group = state
        .directory
        .authz()
        .group(&id)
        .await?
        .ok_or_else(|| ScimError::not_found("Group", &id))?;
    let mut resources = group_resources(&state.directory, &[group]).await?;
    Ok(scim_json(StatusCode::OK, resources.remove(0)))
}

async fn list_service_principals(
    State(state): State<ScimState>,
    cx: RequestContext,
    Query(params): Query<ListParams>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let accounts = state.directory.authz().service_principals().await?;
    let groups = state.directory.authz().groups().await?;
    let resources = accounts
        .iter()
        .map(|a| {
            service_principal_resource(a, &groups_of(&groups, SubjectKind::ServiceAccount, &a.id))
        })
        .collect();
    list_response(resources, &params)
}

async fn get_service_principal(
    State(state): State<ScimState>,
    cx: RequestContext,
    Path(id): Path<String>,
) -> ScimResult {
    require_caller(&state, &cx)?;
    let account = state
        .directory
        .authz()
        .service_principal(&id)
        .await?
        .ok_or_else(|| ScimError::not_found("ServicePrincipal", &id))?;
    let groups = state.directory.authz().groups().await?;
    Ok(scim_json(
        StatusCode::OK,
        service_principal_resource(
            &account,
            &groups_of(&groups, SubjectKind::ServiceAccount, &account.id),
        ),
    ))
}

/// The caller: a verified user or service account by id, a named caller by
/// id or name (else by its name alone); 401 for an anonymous one.
async fn me(State(state): State<ScimState>, cx: RequestContext) -> ScimResult {
    let directory = &state.directory;
    let groups = directory.authz().groups().await?;
    #[cfg(feature = "olai-auth")]
    if let Some(principal) = cx.verified_principal() {
        match &principal.kind {
            olai_auth::PrincipalKind::User { sub, email, .. } => {
                let user = match directory.users().user(sub).await? {
                    Some(user) => user,
                    None => User {
                        id: sub.clone(),
                        user_name: email.clone().unwrap_or_else(|| sub.clone()),
                        display_name: None,
                        emails: email.iter().cloned().collect(),
                        active: true,
                    },
                };
                let of = groups_of(&groups, SubjectKind::User, &user.id);
                return Ok(scim_json(StatusCode::OK, user_resource(&user, &of)));
            }
            olai_auth::PrincipalKind::ServiceAccount { id, name, .. } => {
                let account = match directory.authz().service_principal(id).await? {
                    Some(account) => account,
                    None => ServicePrincipal {
                        id: id.clone(),
                        name: name.clone(),
                        description: None,
                        active: true,
                    },
                };
                let of = groups_of(&groups, SubjectKind::ServiceAccount, &account.id);
                return Ok(scim_json(
                    StatusCode::OK,
                    service_principal_resource(&account, &of),
                ));
            }
        }
    }
    let Principal::User(name) = cx.recipient() else {
        return Err(ScimError::new(StatusCode::UNAUTHORIZED, "not signed in"));
    };
    let user = match directory.users().user(name).await? {
        Some(user) => Some(user),
        None => directory.users().user_by_name(name).await?,
    };
    let user = user.unwrap_or_else(|| User {
        id: name.clone(),
        user_name: name.clone(),
        display_name: None,
        emails: name
            .contains('@')
            .then(|| name.clone())
            .into_iter()
            .collect(),
        active: true,
    });
    let of = groups_of(&groups, SubjectKind::User, &user.id);
    Ok(scim_json(StatusCode::OK, user_resource(&user, &of)))
}

#[cfg(test)]
mod tests;
