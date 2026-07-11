//! Hand-written client for the UC Delta REST API (`/delta/v1/...`).
//!
//! The Delta API is a standalone REST protocol (not a generated resource API),
//! so — like [`crate::temporary_credentials`] — it is hand-maintained. The wire
//! DTOs are shared with the server via
//! [`unitycatalog_delta_api::models`]. This client covers the full
//! `delta.yaml` surface: catalog config negotiation, the catalog-managed table
//! lifecycle (`createStagingTable`, `createTable`, `loadTable`, `updateTable`,
//! `deleteTable`, `tableExists`, `renameTable`), credential vending
//! (`getTableCredentials`, `getStagingTableCredentials`,
//! `getTemporaryPathCredentials`), and commit-metrics reporting (`reportMetrics`).

use olai_http::CloudClient;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use unitycatalog_delta_api::models::{
    DeltaCatalogConfig, DeltaCreateStagingTableRequest, DeltaCreateTableRequest,
    DeltaCredentialOperation, DeltaCredentialsResponse, DeltaLoadTableResponse,
    DeltaRenameTableRequest, DeltaReportMetricsRequest, DeltaStagingTableResponse,
    DeltaUpdateTableRequest,
};
use url::Url;

use crate::Result;

/// Percent-encode a single path segment so names containing `/ ? # space ..`
/// route to the intended resource instead of being interpreted as path
/// structure. We encode every non-alphanumeric byte (a superset of the RFC 3986
/// path-segment reserved set), which is safe because each call encodes exactly
/// one segment that is then joined with literal `/` separators.
fn encode_segment(segment: &str) -> String {
    utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string()
}

/// The wire string for a credential operation as used in `operation` query params.
fn operation_param(operation: DeltaCredentialOperation) -> &'static str {
    match operation {
        DeltaCredentialOperation::Read => "READ",
        DeltaCredentialOperation::ReadWrite => "READ_WRITE",
    }
}

/// Client for the `/delta/v1/` Delta REST API.
///
/// Constructed from a [`CloudClient`] (carrying auth) and the Unity Catalog base
/// URL. Prefer [`UnityCatalogClient::delta_v1`](crate::UnityCatalogClient::delta_v1).
#[derive(Clone)]
pub struct DeltaV1Client {
    client: CloudClient,
    base_url: Url,
}

impl DeltaV1Client {
    /// Create a new Delta v1 client. `base_url` is normalized to end in `/` so it
    /// joins cleanly with the relative endpoint paths.
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }

    /// Build a `/delta/v1/...` URL from already-encoded path components.
    fn url(&self, rest: &str) -> Result<Url> {
        Ok(self.base_url.join(&format!("delta/v1/{rest}"))?)
    }

    /// Get catalog configuration and supported endpoints —
    /// `GET /delta/v1/config?catalog={catalog}&protocol-versions={versions}`.
    ///
    /// This is the protocol-negotiation entry point: the client advertises the
    /// highest protocol versions it supports per major version (`protocol_versions`,
    /// e.g. `"1.1,2.3"`) and the server returns the endpoints it supports for the
    /// newest mutually supported version. Used to discover whether the
    /// `/delta/v1` surface is available before falling back to the legacy API.
    pub async fn get_config(
        &self,
        catalog: &str,
        protocol_versions: &str,
    ) -> Result<DeltaCatalogConfig> {
        let url = self.url("config")?;
        let response = self
            .client
            .get(url)
            .query(&[
                ("catalog", catalog),
                ("protocol-versions", protocol_versions),
            ])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Create a staging table — `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables`.
    ///
    /// Allocates an immutable table id and a managed storage `location`, and
    /// advertises the catalog-managed contract (required/suggested protocol +
    /// properties) plus `READ_WRITE` credentials for writing the initial
    /// `_delta_log/0.json`. The catalog/schema are taken from the path; the
    /// request body carries only the table name.
    pub async fn create_staging_table(
        &self,
        catalog: &str,
        schema: &str,
        request: &DeltaCreateStagingTableRequest,
    ) -> Result<DeltaStagingTableResponse> {
        let url = self.url(&format!(
            "catalogs/{}/schemas/{}/staging-tables",
            encode_segment(catalog),
            encode_segment(schema),
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Finalize a managed table from its staging reservation —
    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables`.
    ///
    /// Called after the client has written `_delta_log/0.json` (carrying the
    /// required features/properties, including `io.unitycatalog.tableId`) to the
    /// staging `location`. The server validates the contract and registers the
    /// table at version 0.
    pub async fn create_table(
        &self,
        catalog: &str,
        schema: &str,
        request: &DeltaCreateTableRequest,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.url(&format!(
            "catalogs/{}/schemas/{}/tables",
            encode_segment(catalog),
            encode_segment(schema),
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Load a Delta table's metadata, unbackfilled CCv2 commits, and latest
    /// ratified version — `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// For a catalog-managed (MANAGED) table the response carries the `commits`
    /// log tail and `latest_table_version` a reader needs to materialize the
    /// catalog's ratified snapshot.
    pub async fn load_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.table_url(catalog, schema, table, "")?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Update a table — `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// This is the catalog-managed commit surface: the `add-commit` action
    /// proposes a staged commit (the server ratifies iff `version == last + 1`,
    /// returning `409` on conflict), and `set-latest-backfilled-version` notifies
    /// the catalog that commits up to a version have been published into
    /// `_delta_log/`. Metadata/property/schema changes ride the same call.
    pub async fn update_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
        request: &DeltaUpdateTableRequest,
    ) -> Result<DeltaLoadTableResponse> {
        let url = self.table_url(catalog, schema, table, "")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Delete a table — `DELETE /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// The server responds `204 No Content` on success.
    pub async fn delete_table(&self, catalog: &str, schema: &str, table: &str) -> Result<()> {
        let url = self.table_url(catalog, schema, table, "")?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        Ok(())
    }

    /// Check whether a table exists —
    /// `HEAD /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`.
    ///
    /// Returns `Ok(true)` on `204`, `Ok(false)` on `404`, and an error for any
    /// other non-success status. A HEAD response carries no body, so a `404` here
    /// is mapped directly rather than parsed as a Delta error envelope.
    pub async fn table_exists(&self, catalog: &str, schema: &str, table: &str) -> Result<bool> {
        let url = self.table_url(catalog, schema, table, "")?;
        let response = self.client.head(url).send().await?;
        let status = response.status();
        if status.is_success() {
            Ok(true)
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Ok(false)
        } else {
            Err(crate::error::parse_delta_error_response(response).await)
        }
    }

    /// Rename a table within the same catalog and schema —
    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename`.
    ///
    /// Cross-schema and cross-catalog moves are not supported. The server responds
    /// `204 No Content` on success.
    pub async fn rename_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
        request: &DeltaRenameTableRequest,
    ) -> Result<()> {
        let url = self.table_url(catalog, schema, table, "/rename")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        Ok(())
    }

    /// Vend temporary credentials for a table's data —
    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials?operation={op}`.
    ///
    /// `operation` scopes the credential to `READ` or `READ_WRITE` access.
    pub async fn get_table_credentials(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
        operation: DeltaCredentialOperation,
    ) -> Result<DeltaCredentialsResponse> {
        let url = self.table_url(catalog, schema, table, "/credentials")?;
        let response = self
            .client
            .get(url)
            .query(&[("operation", operation_param(operation))])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Vend `READ_WRITE` credentials for a staging table identified by its UUID —
    /// `GET /delta/v1/staging-tables/{table_id}/credentials`.
    ///
    /// A staging table is globally identified by its UUID (no catalog/schema). This
    /// is how staging-phase credentials are refreshed while the client writes the
    /// initial commit.
    pub async fn get_staging_table_credentials(
        &self,
        table_id: &str,
    ) -> Result<DeltaCredentialsResponse> {
        let url = self.url(&format!(
            "staging-tables/{}/credentials",
            encode_segment(table_id),
        ))?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Vend temporary credentials for a storage path (for creating a new external
    /// table) — `GET /delta/v1/temporary-path-credentials?location={location}&operation={op}`.
    ///
    /// The path is not yet part of any catalog/namespace, so it is passed as a
    /// query parameter rather than a path segment.
    pub async fn get_temporary_path_credentials(
        &self,
        location: &str,
        operation: DeltaCredentialOperation,
    ) -> Result<DeltaCredentialsResponse> {
        let url = self.url("temporary-path-credentials")?;
        let response = self
            .client
            .get(url)
            .query(&[
                ("location", location),
                ("operation", operation_param(operation)),
            ])
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }

    /// Report commit metrics (telemetry) for a table —
    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics`.
    ///
    /// The path `{table}` and the body `table-id` must identify the same table.
    /// The server responds `204 No Content` on success.
    pub async fn report_metrics(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
        request: &DeltaReportMetricsRequest,
    ) -> Result<()> {
        let url = self.table_url(catalog, schema, table, "/metrics")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_delta_error_response(response).await);
        }
        Ok(())
    }

    /// Build a `catalogs/{c}/schemas/{s}/tables/{t}{suffix}` URL with each name
    /// percent-encoded. `suffix` is a literal sub-path (e.g. `"/rename"`) or empty.
    fn table_url(&self, catalog: &str, schema: &str, table: &str, suffix: &str) -> Result<Url> {
        self.url(&format!(
            "catalogs/{}/schemas/{}/tables/{}{suffix}",
            encode_segment(catalog),
            encode_segment(schema),
            encode_segment(table),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use unitycatalog_delta_api::models::DeltaErrorType;

    fn test_client(server: &Server) -> DeltaV1Client {
        let base = Url::parse(&server.url()).unwrap();
        DeltaV1Client::new(CloudClient::new_unauthenticated(), base)
    }

    /// A minimal `DeltaLoadTableResponse` JSON body for happy-path assertions.
    fn load_table_body() -> &'static str {
        r#"{"metadata":{"etag":"e","table-type":"MANAGED","table-uuid":"u",
            "location":"s3://b/t","created-time":0,"updated-time":0,
            "columns":{"type":"struct","fields":[]},"properties":{}}}"#
    }

    #[tokio::test]
    async fn load_table_maps_delta_not_found() {
        let mut server = Server::new_async().await;
        let m = server
            .mock("GET", "/delta/v1/catalogs/c/schemas/s/tables/t")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"error":{"message":"no table","type":"NoSuchTableException","code":404}}"#,
            )
            .create_async()
            .await;

        let err = test_client(&server)
            .load_table("c", "s", "t")
            .await
            .unwrap_err();
        m.assert_async().await;
        assert!(err.is_not_found());
        assert!(matches!(
            err,
            crate::Error::Delta(ref model)
                if model.error_type == DeltaErrorType::NoSuchTableException
        ));
    }

    #[tokio::test]
    async fn update_table_maps_commit_conflict() {
        let mut server = Server::new_async().await;
        let m = server
            .mock("POST", "/delta/v1/catalogs/c/schemas/s/tables/t")
            .with_status(409)
            .with_body(
                r#"{"error":{"message":"conflict","type":"CommitVersionConflictException","code":409}}"#,
            )
            .create_async()
            .await;

        let req = DeltaUpdateTableRequest {
            requirements: vec![],
            updates: vec![],
        };
        let err = test_client(&server)
            .update_table("c", "s", "t", &req)
            .await
            .unwrap_err();
        m.assert_async().await;
        assert!(err.is_commit_conflict());
    }

    #[tokio::test]
    async fn url_segments_are_percent_encoded() {
        let mut server = Server::new_async().await;
        // A schema with a space and a table with a slash must be encoded so they
        // route to a single resource rather than extra path structure.
        let m = server
            .mock(
                "GET",
                "/delta/v1/catalogs/c/schemas/my%20schema/tables/weird%2Fname",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(load_table_body())
            .create_async()
            .await;

        let resp = test_client(&server)
            .load_table("c", "my schema", "weird/name")
            .await
            .unwrap();
        m.assert_async().await;
        assert_eq!(resp.metadata.location, "s3://b/t");
    }

    #[tokio::test]
    async fn table_exists_true_false() {
        let mut server = Server::new_async().await;
        let exists = server
            .mock("HEAD", "/delta/v1/catalogs/c/schemas/s/tables/yes")
            .with_status(204)
            .create_async()
            .await;
        let missing = server
            .mock("HEAD", "/delta/v1/catalogs/c/schemas/s/tables/no")
            .with_status(404)
            .create_async()
            .await;

        let client = test_client(&server);
        assert!(client.table_exists("c", "s", "yes").await.unwrap());
        assert!(!client.table_exists("c", "s", "no").await.unwrap());
        exists.assert_async().await;
        missing.assert_async().await;
    }

    #[tokio::test]
    async fn get_config_sends_query_params() {
        let mut server = Server::new_async().await;
        let m = server
            .mock("GET", "/delta/v1/config")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("catalog".into(), "main".into()),
                mockito::Matcher::UrlEncoded("protocol-versions".into(), "1.1,2.3".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"endpoints":["GET /v1/config"],"protocol-version":"1.0"}"#)
            .create_async()
            .await;

        let cfg = test_client(&server)
            .get_config("main", "1.1,2.3")
            .await
            .unwrap();
        m.assert_async().await;
        assert_eq!(cfg.protocol_version, "1.0");
    }

    #[tokio::test]
    async fn get_table_credentials_sends_operation() {
        let mut server = Server::new_async().await;
        let m = server
            .mock("GET", "/delta/v1/catalogs/c/schemas/s/tables/t/credentials")
            .match_query(mockito::Matcher::UrlEncoded(
                "operation".into(),
                "READ_WRITE".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"storage-credentials":[]}"#)
            .create_async()
            .await;

        let resp = test_client(&server)
            .get_table_credentials("c", "s", "t", DeltaCredentialOperation::ReadWrite)
            .await
            .unwrap();
        m.assert_async().await;
        assert!(resp.storage_credentials.is_empty());
    }

    #[tokio::test]
    async fn rename_and_delete_and_metrics_no_content() {
        let mut server = Server::new_async().await;
        let rename = server
            .mock("POST", "/delta/v1/catalogs/c/schemas/s/tables/t/rename")
            .with_status(204)
            .create_async()
            .await;
        let delete = server
            .mock("DELETE", "/delta/v1/catalogs/c/schemas/s/tables/t")
            .with_status(204)
            .create_async()
            .await;
        let metrics = server
            .mock("POST", "/delta/v1/catalogs/c/schemas/s/tables/t/metrics")
            .with_status(204)
            .create_async()
            .await;

        let client = test_client(&server);
        client
            .rename_table(
                "c",
                "s",
                "t",
                &DeltaRenameTableRequest {
                    new_name: "t2".into(),
                },
            )
            .await
            .unwrap();
        client.delete_table("c", "s", "t").await.unwrap();
        client
            .report_metrics(
                "c",
                "s",
                "t",
                &DeltaReportMetricsRequest {
                    table_id: "u".into(),
                    report: None,
                },
            )
            .await
            .unwrap();
        rename.assert_async().await;
        delete.assert_async().await;
        metrics.assert_async().await;
    }

    #[tokio::test]
    async fn temporary_path_credentials_query() {
        let mut server = Server::new_async().await;
        let m = server
            .mock("GET", "/delta/v1/temporary-path-credentials")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("location".into(), "s3://bucket/path".into()),
                mockito::Matcher::UrlEncoded("operation".into(), "READ".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"storage-credentials":[]}"#)
            .create_async()
            .await;

        test_client(&server)
            .get_temporary_path_credentials("s3://bucket/path", DeltaCredentialOperation::Read)
            .await
            .unwrap();
        m.assert_async().await;
    }
}
