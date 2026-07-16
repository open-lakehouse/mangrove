//! Axum routers for the Delta Sharing and Open Sharing REST surfaces.
//!
//! [`get_router`] serves the Delta Sharing tabular surface (shares / schemas /
//! tables / version / metadata / query); [`open_sharing_router`] adds the Open
//! Sharing asset routes (volumes, agent skills) plus the not-yet-implemented
//! protocol additions (change data feed, asynchronous queries).
//!
//! The discovery + asset routes bind to the generated route functions; the
//! version / metadata / query routes bind to the hand-written functions below,
//! whose newline-delimited-JSON response contract the generated, JSON-only route
//! functions do not model.
//!
//! # Spec-gap routes
//!
//! The change-data-feed (`/changes`) and asynchronous-query
//! (`POST /queries/{id}`) routes exist so the surface matches the evolved Delta
//! Sharing protocol, but their handlers return `501 Not Implemented`. The CDF
//! route reuses [`QueryTableRequest`] — whose proto already carries the CDF
//! `starting_version` / `ending_version` fields — rather than a dedicated
//! request type, since no serving logic consumes it yet.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{Router, get, post};

use unitycatalog_sharing_client::models::open_sharing::v1::*;

use crate::codegen::sharing::{self, SharingHandler};
use crate::codegen::sharing_skill::{self, SharingSkillHandler};
use crate::codegen::sharing_volume::{self, SharingVolumeHandler};
use crate::error::{Error, Result};
use crate::handler::SharingQueryHandler;

/// Response header advertising the Delta Sharing capabilities this server
/// supports. The query path currently emits responses in `parquet` format.
const DELTA_SHARING_CAPABILITIES: &str = "delta-sharing-capabilities";
const DELTA_SHARING_CAPABILITIES_VALUE: &str = "responseformat=parquet";
const CONTENT_TYPE: &str = "content-type";

/// The tabular Delta Sharing routes (shares / schemas / tables / version /
/// metadata / query).
///
/// Shared verbatim between the Delta Sharing and Open Sharing mounts. The
/// discovery routes bind to the generated [`SharingHandler`] route functions; the
/// three NDJSON query routes bind to the hand-written functions below.
fn tabular_routes<T, Cx>() -> Router<T>
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route("/shares", get(sharing::server::list_shares::<T, Cx>))
        .route("/shares/{name}", get(sharing::server::get_share::<T, Cx>))
        .route(
            "/shares/{share}/schemas",
            get(sharing::server::list_schemas::<T, Cx>),
        )
        .route(
            "/shares/{share}/all-tables",
            get(sharing::server::list_all_tables::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{name}/tables",
            get(sharing::server::list_tables::<T, Cx>),
        )
        // Hand-written NDJSON query path (not part of the generated service).
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/version",
            get(get_table_version::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/metadata",
            get(get_table_metadata::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/query",
            post(get_table_query::<T, Cx>),
        )
        // Recently added protocol endpoints: types + routes exist for spec
        // coverage; the handlers return 501 until the serving path is implemented.
        .route(
            "/shares/{share}/schemas/{schema}/tables/{name}/changes",
            get(get_table_changes::<T, Cx>),
        )
        .route("/queries/{query_id}", post(poll_query::<T, Cx>))
}

/// The Open-Sharing-only asset routes (volumes, agent skills), bound to the
/// generated per-asset route functions.
fn asset_routes<T, Cx>() -> Router<T>
where
    T: SharingVolumeHandler<Cx> + SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route(
            "/shares/{share}/all-volumes",
            get(sharing_volume::server::list_all_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes",
            get(sharing_volume::server::list_volumes::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}",
            get(sharing_volume::server::get_volume::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/volumes/{name}/temporary-volume-credentials",
            post(sharing_volume::server::generate_temporary_volume_credentials::<T, Cx>),
        )
        .route(
            "/shares/{share}/all-skills",
            get(sharing_skill::server::list_all_skills::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills",
            get(sharing_skill::server::list_skills::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}",
            get(sharing_skill::server::get_skill::<T, Cx>),
        )
        .route(
            "/shares/{share}/schemas/{schema}/skills/{name}/temporary-skill-credentials",
            post(sharing_skill::server::generate_temporary_skill_credentials::<T, Cx>),
        )
}

/// Create a [`Router`] for the **Delta Sharing** REST API — the tabular surface
/// only, preserved for wire-compatibility with existing Delta Sharing clients.
pub fn get_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx> + SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    tabular_routes::<T, Cx>().with_state(state)
}

/// Create a [`Router`] for the **Open Sharing** REST API: the tabular surface
/// plus the storage-backed asset routes (volumes, agent skills).
pub fn open_sharing_router<T, Cx>(state: T) -> Router
where
    T: SharingHandler<Cx>
        + SharingQueryHandler<Cx>
        + SharingVolumeHandler<Cx>
        + SharingSkillHandler<Cx>
        + Clone
        + Send
        + Sync
        + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    tabular_routes::<T, Cx>()
        .merge(asset_routes::<T, Cx>())
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Hand-written NDJSON query routes (version / metadata / query) + the 501 gap
// routes (changes / queries poll).
// ---------------------------------------------------------------------------

async fn get_table_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableVersionRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_version(request, context).await?;
    Response::builder()
        .header("Delta-Table-Version", result.version)
        .body(Body::empty())
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_metadata<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableMetadataRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_metadata(request, context).await?;
    Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))
}

async fn get_table_query<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: QueryTableRequest,
) -> Result<Response>
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.query_table(request, context).await?;
    Response::builder()
        .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
        .body(Body::from(result))
        .map_err(|e| Error::generic(e.to_string()))
}

/// Change data feed. **Not implemented** (501). A GET with `startingVersion` /
/// `endingVersion` query params in the spec; here it only extracts the table path
/// and hands a minimally-populated request to the handler, which returns 501.
async fn get_table_changes<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((share, schema, name)): Path<(String, String, String)>,
) -> Response
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let request = QueryTableRequest {
        share,
        schema,
        name,
        ..Default::default()
    };
    match handler.get_table_changes(request, context).await {
        Ok(body) => Response::builder()
            .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
            .header(DELTA_SHARING_CAPABILITIES, DELTA_SHARING_CAPABILITIES_VALUE)
            .body(Body::from(body))
            .unwrap_or_else(|e| Error::generic(e.to_string()).into_response()),
        Err(e) => e.into_response(),
    }
}

/// Poll an asynchronous query by id. **Not implemented** (501).
async fn poll_query<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path(query_id): Path<String>,
) -> Response
where
    T: SharingQueryHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    match handler.poll_query(query_id, context).await {
        Ok(body) => Response::builder()
            .header(CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
            .body(Body::from(body))
            .unwrap_or_else(|e| Error::generic(e.to_string()).into_response()),
        Err(e) => e.into_response(),
    }
}
