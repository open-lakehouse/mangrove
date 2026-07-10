//! Axum router for the UC Delta REST API (`/delta/v1/...`).
//!
//! [`get_router`] mounts every operation; each per-operation handler extracts the
//! request, calls [`DeltaApiHandler`], and serializes the response. Generic over
//! the handler `T` and the context `Cx` (extracted via `FromRequestParts<T>`), so
//! each server plugs in its own state + context type.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{Router, get, post};
use serde::Deserialize;

use crate::error::DeltaApiResult as DeltaResult;
use crate::handler::{DeltaApiHandler, GetConfigQuery, SchemaPath, TablePath};
use crate::models::*;

/// Create a [`Router`] for the Delta REST API. Routes match `openapi/delta.yaml`.
pub fn get_router<T, Cx>(state: T) -> Router
where
    T: DeltaApiHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    Router::new()
        .route("/delta/v1/config", get(get_config::<T, Cx>))
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables",
            post(create_staging_table::<T, Cx>),
        )
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/tables",
            post(create_table::<T, Cx>),
        )
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
            get(load_table::<T, Cx>)
                .post(update_table::<T, Cx>)
                .delete(delete_table::<T, Cx>)
                .head(table_exists::<T, Cx>),
        )
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename",
            post(rename_table::<T, Cx>),
        )
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials",
            get(get_table_credentials::<T, Cx>),
        )
        .route(
            "/delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics",
            post(report_metrics::<T, Cx>),
        )
        .route(
            "/delta/v1/staging-tables/{table_id}/credentials",
            get(get_staging_table_credentials::<T, Cx>),
        )
        .route(
            "/delta/v1/temporary-path-credentials",
            get(get_temporary_path_credentials::<T, Cx>),
        )
        .with_state(state)
}

// ----- Query parameter deserialization helpers -------------------------------

#[derive(Debug, Deserialize)]
struct GetConfigParams {
    catalog: String,
    #[serde(rename = "protocol-versions")]
    protocol_versions: String,
}

#[derive(Debug, Deserialize)]
struct OperationParam {
    operation: DeltaCredentialOperation,
}

#[derive(Debug, Deserialize)]
struct PathCredentialParams {
    location: String,
    operation: DeltaCredentialOperation,
}

// ----- Handlers --------------------------------------------------------------

async fn get_config<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Query(params): Query<GetConfigParams>,
) -> DeltaResult<axum::Json<DeltaCatalogConfig>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let query = GetConfigQuery {
        catalog: params.catalog,
        protocol_versions: params.protocol_versions,
    };
    Ok(axum::Json(handler.get_config(query, context).await?))
}

async fn create_staging_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema)): Path<(String, String)>,
    axum::Json(request): axum::Json<DeltaCreateStagingTableRequest>,
) -> DeltaResult<axum::Json<DeltaStagingTableResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = SchemaPath { catalog, schema };
    Ok(axum::Json(
        handler.create_staging_table(path, request, context).await?,
    ))
}

async fn create_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema)): Path<(String, String)>,
    axum::Json(request): axum::Json<DeltaCreateTableRequest>,
) -> DeltaResult<axum::Json<DeltaLoadTableResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = SchemaPath { catalog, schema };
    Ok(axum::Json(
        handler.create_table(path, request, context).await?,
    ))
}

async fn load_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
) -> DeltaResult<axum::Json<DeltaLoadTableResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    Ok(axum::Json(handler.load_table(path, context).await?))
}

async fn update_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
    axum::Json(request): axum::Json<DeltaUpdateTableRequest>,
) -> DeltaResult<axum::Json<DeltaLoadTableResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    Ok(axum::Json(
        handler.update_table(path, request, context).await?,
    ))
}

async fn delete_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
) -> DeltaResult<StatusCode>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    handler.delete_table(path, context).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn table_exists<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
) -> DeltaResult<StatusCode>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    handler.table_exists(path, context).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn rename_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
    axum::Json(request): axum::Json<DeltaRenameTableRequest>,
) -> DeltaResult<StatusCode>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    handler.rename_table(path, request, context).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_table_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
    Query(params): Query<OperationParam>,
) -> DeltaResult<axum::Json<DeltaCredentialsResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    Ok(axum::Json(
        handler
            .get_table_credentials(path, params.operation, context)
            .await?,
    ))
}

async fn report_metrics<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path((catalog, schema, table)): Path<(String, String, String)>,
    axum::Json(request): axum::Json<DeltaReportMetricsRequest>,
) -> DeltaResult<StatusCode>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let path = TablePath {
        catalog,
        schema,
        table,
    };
    handler.report_metrics(path, request, context).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_staging_table_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Path(table_id): Path<String>,
) -> DeltaResult<axum::Json<DeltaCredentialsResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(
        handler
            .get_staging_table_credentials(table_id, context)
            .await?,
    ))
}

async fn get_temporary_path_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    Query(params): Query<PathCredentialParams>,
) -> DeltaResult<axum::Json<DeltaCredentialsResponse>>
where
    T: DeltaApiHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    Ok(axum::Json(
        handler
            .get_temporary_path_credentials(params.location, params.operation, context)
            .await?,
    ))
}
