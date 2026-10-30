//! The provider admin API, in the Unity Catalog REST shape
//! (`/api/2.1/unity-catalog/…`), behind provider bearer tokens.
//!
//! - `shares`: create / list / get / update (add, remove or update data objects)
//!   / delete, and `shares/{name}/permissions` (grant `SELECT` to recipients);
//! - `recipients`: create (issuing a token + activation link) / list / get /
//!   delete, and `recipients/{name}/rotate-token`;
//! - `credentials` and `external-locations`: the stored storage credentials local
//!   access vends from (credentials are sealed and never returned).
//!
//! With no provider token configured the admin API rejects every request.
//!
//! The store keeps shares as catalog-independent [`ShareObject`]s; this module
//! converts them to and from UC `DataObject`s at the API boundary.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use olai_credential_vending::RegisteredCredential;
use olai_storage_credentials::StorageLocation;
use serde_json::json;
use unitycatalog_common::models::credentials::v1::Credential;
use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
use unitycatalog_common::models::recipients::v1::{
    AuthenticationType, CreateRecipientRequest, ListRecipientsResponse, Recipient, RecipientToken,
};
use unitycatalog_common::models::shares::v1::{
    Action, CreateShareRequest, DataObject, DataObjectType, GetPermissionsResponse,
    ListSharesResponse, PrivilegeAssignment, Share, UpdatePermissionsRequest,
    UpdatePermissionsResponse, UpdateShareRequest,
};

use crate::access::credential_aad;
use crate::catalog::shared_as;
use crate::recipients::{activation_url, issue_token};
use crate::secrets::Secrets;
use crate::source::split_name;
use crate::store::{
    ExternalLocationRecord, SealedCredential, ShareObject, ShareRecord, ShareStore, TokenRecord,
    now_millis,
};
use crate::tokens::token_hash;
use crate::{Error, Result};

/// Where the admin API is mounted.
pub const ADMIN_PREFIX: &str = "/api/2.1/unity-catalog";

/// The only privilege a share grants.
const SELECT: &str = "SELECT";

/// State shared by the admin handlers.
#[derive(Clone)]
pub struct ProviderState {
    pub store: Arc<dyn ShareStore>,
    pub secrets: Secrets,
    pub public_url: String,
    /// Digests of the accepted provider tokens.
    pub provider_tokens: Arc<Vec<Vec<u8>>>,
}

/// The admin router, with provider-token authentication.
pub fn router(state: ProviderState) -> axum::Router {
    axum::Router::new()
        .route("/shares", get(list_shares).post(create_share))
        .route(
            "/shares/{name}",
            get(get_share).patch(update_share).delete(delete_share),
        )
        .route(
            "/shares/{name}/permissions",
            get(get_permissions).patch(update_permissions),
        )
        .route("/recipients", get(list_recipients).post(create_recipient))
        .route(
            "/recipients/{name}",
            get(get_recipient).delete(delete_recipient),
        )
        .route("/recipients/{name}/rotate-token", post(rotate_token))
        .route(
            "/credentials",
            get(list_credentials).post(create_credential),
        )
        .route(
            "/credentials/{name}",
            axum::routing::delete(delete_credential),
        )
        .route(
            "/external-locations",
            get(list_external_locations).post(create_external_location),
        )
        .route(
            "/external-locations/{name}",
            axum::routing::delete(delete_external_location),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_provider,
        ))
        .with_state(state)
}

async fn require_provider(
    State(state): State<ProviderState>,
    request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim);
    match token {
        Some(token) if state.provider_tokens.contains(&token_hash(token)) => {
            next.run(request).await
        }
        _ => Error::Unauthenticated.into_response(),
    }
}

// -- shares -------------------------------------------------------------------

/// A UC `DataObject` as the store's [`ShareObject`]. Both use the UC field
/// names, so the conversion goes through JSON and keeps every field.
fn share_object(object: &DataObject) -> Result<ShareObject> {
    serde_json::from_value(serde_json::to_value(object)?).map_err(|_| {
        Error::invalid_argument(format!(
            "data object '{}' needs a data_object_type of TABLE, SCHEMA, VOLUME or AGENT_SKILL",
            object.name
        ))
    })
}

/// A stored [`ShareObject`] as a UC `DataObject`.
fn data_object(object: &ShareObject) -> Result<DataObject> {
    Ok(serde_json::from_value(serde_json::to_value(object)?)?)
}

fn share(record: ShareRecord) -> Result<Share> {
    Ok(Share {
        id: Some(record.id),
        name: record.name,
        objects: record
            .objects
            .iter()
            .map(data_object)
            .collect::<Result<_>>()?,
        comment: record.comment,
        created_at: Some(record.created_at),
        updated_at: Some(record.updated_at),
        ..Default::default()
    })
}

async fn create_share(
    State(state): State<ProviderState>,
    Json(request): Json<CreateShareRequest>,
) -> Result<Json<Share>> {
    identifier(&request.name)?;
    Ok(Json(share(
        state
            .store
            .create_share(&request.name, request.comment)
            .await?,
    )?))
}

async fn list_shares(State(state): State<ProviderState>) -> Result<Json<ListSharesResponse>> {
    Ok(Json(ListSharesResponse {
        shares: state
            .store
            .list_shares()
            .await?
            .into_iter()
            .map(share)
            .collect::<Result<_>>()?,
        ..Default::default()
    }))
}

async fn get_share(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<Json<Share>> {
    Ok(Json(share(state.store.get_share(&name).await?)?))
}

async fn delete_share(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.store.delete_share(&name).await?;
    Ok(StatusCode::OK)
}

/// Check `name` is a plain identifier (non-empty, no `.` or `/`).
fn identifier(name: &str) -> Result<()> {
    if name.is_empty() || name.contains(['.', '/']) {
        return Err(Error::invalid_argument(format!(
            "'{name}' is not a valid name (non-empty, without '.' or '/')"
        )));
    }
    Ok(())
}

/// Check a data object is well formed for its type.
fn validate_object(object: &DataObject) -> Result<()> {
    match object.data_object_type.as_known() {
        Some(DataObjectType::TABLE | DataObjectType::VOLUME | DataObjectType::AGENT_SKILL) => {
            split_name::<3>(&object.name)?;
            shared_as(&share_object(object)?)?;
        }
        Some(DataObjectType::SCHEMA) => {
            split_name::<2>(&object.name)?;
            if let Some(shared) = &object.shared_as {
                identifier(shared)?;
            }
        }
        _ => {
            return Err(Error::invalid_argument(format!(
                "data object '{}' needs a data_object_type of TABLE, SCHEMA, VOLUME or AGENT_SKILL",
                object.name
            )));
        }
    }
    Ok(())
}

async fn update_share(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
    Json(request): Json<UpdateShareRequest>,
) -> Result<Json<Share>> {
    if request.new_name.is_some() {
        return Err(Error::invalid_argument("renaming shares is not supported"));
    }
    let current = state.store.get_share(&name).await?;
    let mut objects = current
        .objects
        .iter()
        .map(data_object)
        .collect::<Result<Vec<_>>>()?;
    for update in request.updates {
        let object = update
            .data_object
            .as_option()
            .cloned()
            .ok_or_else(|| Error::invalid_argument("a share update needs a data_object"))?;
        let position = objects.iter().position(|o| o.name == object.name);
        match (update.action.as_known(), position) {
            (Some(Action::ADD), None) => {
                validate_object(&object)?;
                objects.push(DataObject {
                    added_at: Some(now_millis()),
                    ..object
                });
            }
            (Some(Action::ADD), Some(_)) => {
                return Err(Error::invalid_argument(format!(
                    "'{}' is already in the share",
                    object.name
                )));
            }
            (Some(Action::UPDATE), Some(i)) => {
                validate_object(&object)?;
                objects[i] = DataObject {
                    added_at: objects[i].added_at,
                    ..object
                };
            }
            (Some(Action::REMOVE), Some(i)) => {
                objects.remove(i);
            }
            (Some(Action::UPDATE | Action::REMOVE), None) => {
                return Err(Error::invalid_argument(format!(
                    "'{}' is not in the share",
                    object.name
                )));
            }
            _ => return Err(Error::invalid_argument("unknown share update action")),
        }
    }
    // Two objects may not be exposed under the same shared name.
    let mut exposed = std::collections::HashSet::new();
    for object in objects
        .iter()
        .filter(|o| o.data_object_type.as_known() != Some(DataObjectType::SCHEMA))
    {
        let key = (
            object.data_object_type.to_i32(),
            shared_as(&share_object(object)?)?,
        );
        if !exposed.insert(key) {
            return Err(Error::invalid_argument(format!(
                "'{}' would be shared under a name already in use",
                object.name
            )));
        }
    }
    let comment = request.comment.or(current.comment);
    let objects = objects
        .iter()
        .map(share_object)
        .collect::<Result<Vec<_>>>()?;
    Ok(Json(share(
        state.store.update_share(&name, objects, comment).await?,
    )?))
}

async fn get_permissions(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<Json<GetPermissionsResponse>> {
    state.store.get_share(&name).await?;
    Ok(Json(GetPermissionsResponse {
        privilege_assignments: assignments(state.store.recipients_of(&name).await?),
        ..Default::default()
    }))
}

fn assignments(recipients: Vec<String>) -> Vec<PrivilegeAssignment> {
    recipients
        .into_iter()
        .map(|principal| PrivilegeAssignment {
            principal,
            privileges: vec![SELECT.to_string()],
            ..Default::default()
        })
        .collect()
}

async fn update_permissions(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
    Json(request): Json<UpdatePermissionsRequest>,
) -> Result<Json<UpdatePermissionsResponse>> {
    for change in &request.changes {
        for privilege in change.add.iter().chain(&change.remove) {
            if !privilege.eq_ignore_ascii_case(SELECT) {
                return Err(Error::invalid_argument(format!(
                    "unsupported privilege '{privilege}'; shares grant SELECT"
                )));
            }
        }
    }
    for change in request.changes {
        if !change.add.is_empty() {
            state.store.grant(&name, &change.principal).await?;
        }
        if !change.remove.is_empty() {
            state.store.revoke(&name, &change.principal).await?;
        }
    }
    Ok(Json(UpdatePermissionsResponse {
        privilege_assignments: assignments(state.store.recipients_of(&name).await?),
        ..Default::default()
    }))
}

// -- recipients ---------------------------------------------------------------

/// A token as shown to the provider; the activation link only until it is used.
pub fn listed_token(public_url: &str, token: &TokenRecord) -> RecipientToken {
    RecipientToken {
        id: token.id.clone(),
        created_at: token.created_at,
        activation_url: activation_url(public_url, token).unwrap_or_default(),
        expiration_time: token.expires_at.unwrap_or_default(),
        ..Default::default()
    }
}

async fn recipient(state: &ProviderState, name: &str) -> Result<Recipient> {
    let record = state.store.get_recipient(name).await?;
    let tokens = state
        .store
        .tokens_for_recipient(&record.id)
        .await?
        .iter()
        .map(|t| listed_token(&state.public_url, t))
        .collect();
    Ok(Recipient {
        id: Some(record.id),
        name: record.name,
        authentication_type: AuthenticationType::TOKEN.into(),
        comment: record.comment,
        created_at: Some(record.created_at),
        tokens,
        ..Default::default()
    })
}

async fn create_recipient(
    State(state): State<ProviderState>,
    Json(request): Json<CreateRecipientRequest>,
) -> Result<Json<Recipient>> {
    match request.authentication_type.as_known() {
        Some(AuthenticationType::TOKEN | AuthenticationType::AUTHENTICATION_TYPE_UNSPECIFIED) => {}
        _ => {
            return Err(Error::invalid_argument(
                "only TOKEN authentication is supported",
            ));
        }
    }
    identifier(&request.name)?;
    let record = state
        .store
        .create_recipient(&request.name, request.comment)
        .await?;
    issue_token(
        state.store.as_ref(),
        &state.secrets,
        &record,
        request.expiration_time,
    )
    .await?;
    Ok(Json(recipient(&state, &record.name).await?))
}

async fn list_recipients(
    State(state): State<ProviderState>,
) -> Result<Json<ListRecipientsResponse>> {
    let mut recipients = Vec::new();
    for record in state.store.list_recipients().await? {
        recipients.push(recipient(&state, &record.name).await?);
    }
    Ok(Json(ListRecipientsResponse {
        recipients,
        ..Default::default()
    }))
}

async fn get_recipient(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<Json<Recipient>> {
    Ok(Json(recipient(&state, &name).await?))
}

async fn delete_recipient(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.store.delete_recipient(&name).await?;
    Ok(StatusCode::OK)
}

/// Replace the recipient's tokens with a fresh one. Existing tokens stop working
/// immediately.
async fn rotate_token(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<Json<Recipient>> {
    let record = state.store.get_recipient(&name).await?;
    let expires_at = state
        .store
        .tokens_for_recipient(&record.id)
        .await?
        .first()
        .and_then(|t| t.expires_at);
    state.store.delete_tokens(&record.id).await?;
    issue_token(state.store.as_ref(), &state.secrets, &record, expires_at).await?;
    Ok(Json(recipient(&state, &name).await?))
}

// -- storage credentials ------------------------------------------------------

async fn create_credential(
    State(state): State<ProviderState>,
    Json(credential): Json<Credential>,
) -> Result<Json<serde_json::Value>> {
    identifier(&credential.name)?;
    // Only the storage credential itself is kept; the catalog metadata is not.
    let storage = RegisteredCredential::try_from(&credential).map_err(|_| {
        Error::invalid_argument(
            "a credential needs one of aws_iam_role, azure_service_principal, \
             azure_managed_identity, azure_storage_key or databricks_gcp_service_account",
        )
    })?;
    let sealed = state
        .secrets
        .seal(
            &credential_aad(&credential.name),
            &serde_json::to_vec(&storage)?,
        )
        .await?;
    state
        .store
        .put_credential(SealedCredential {
            name: credential.name.clone(),
            sealed,
        })
        .await?;
    Ok(Json(json!({ "name": credential.name })))
}

async fn list_credentials(State(state): State<ProviderState>) -> Result<Json<serde_json::Value>> {
    let names: Vec<_> = state
        .store
        .list_credentials()
        .await?
        .into_iter()
        .map(|c| json!({ "name": c.name }))
        .collect();
    Ok(Json(json!({ "credentials": names })))
}

async fn delete_credential(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.store.delete_credential(&name).await?;
    Ok(StatusCode::OK)
}

async fn create_external_location(
    State(state): State<ProviderState>,
    Json(request): Json<CreateExternalLocationRequest>,
) -> Result<Json<serde_json::Value>> {
    identifier(&request.name)?;
    request
        .url
        .parse::<StorageLocation>()
        .map_err(|e| Error::invalid_argument(format!("invalid url: {e}")))?;
    state.store.get_credential(&request.credential_name).await?;
    let location = ExternalLocationRecord {
        name: request.name,
        url: request.url,
        credential_name: request.credential_name,
    };
    state.store.put_external_location(location.clone()).await?;
    Ok(Json(location_json(&location)))
}

fn location_json(location: &ExternalLocationRecord) -> serde_json::Value {
    json!({
        "name": location.name,
        "url": location.url,
        "credential_name": location.credential_name,
    })
}

async fn list_external_locations(
    State(state): State<ProviderState>,
) -> Result<Json<serde_json::Value>> {
    let locations: Vec<_> = state
        .store
        .list_external_locations()
        .await?
        .iter()
        .map(location_json)
        .collect();
    Ok(Json(json!({ "external_locations": locations })))
}

async fn delete_external_location(
    State(state): State<ProviderState>,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    state.store.delete_external_location(&name).await?;
    Ok(StatusCode::OK)
}
