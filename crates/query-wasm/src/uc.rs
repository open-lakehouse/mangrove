//! Minimal Unity Catalog REST client for the browser (wasm-only).
//!
//! Exactly the two calls table resolution needs — `/delta/v1` `loadTable` and
//! `POST temporary-table-credentials` — over reqwest's fetch backend. Requests
//! ride the browser's same-origin credentials (cookies) plus an optional
//! explicit bearer token. This is intentionally NOT `unitycatalog-client`:
//! that crate's `olai-http` transport stack is native, and this standalone
//! workspace only needs a thin projection (see [`crate::resolve`] for the DTOs).

use serde::Deserialize;
use url::Url;

use crate::creds::TemporaryCredential;
use crate::error::{Error, Result};
use crate::resolve::LoadTableResponse;

/// Percent-encode one path segment (matches `unitycatalog-client`'s
/// `encode_segment`: every non-alphanumeric byte, a superset of RFC 3986's
/// reserved set — safe because each segment is joined with literal `/`).
fn encode_segment(segment: &str) -> String {
    percent_encoding::utf8_percent_encode(segment, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// The error envelope both UC surfaces speak, in either nesting.
#[derive(Deserialize)]
struct ErrorEnvelope {
    #[serde(default)]
    error: Option<ErrorBody>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    message: Option<String>,
}

/// Thin UC REST client carrying the base URL and optional bearer auth.
pub struct UcClient {
    client: reqwest::Client,
    base_url: Url,
    bearer: Option<String>,
}

impl UcClient {
    /// Create a client for the UC API rooted at `base_url` (normalized to end
    /// in `/` so relative joins work), e.g. `https://host/api/2.1/unity-catalog`.
    pub fn new(mut base_url: Url, bearer: Option<String>) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self {
            client: reqwest::Client::new(),
            base_url,
            bearer,
        }
    }

    fn request(&self, method: reqwest::Method, url: Url) -> reqwest::RequestBuilder {
        let mut request = self.client.request(method, url);
        if let Some(token) = &self.bearer {
            request = request.bearer_auth(token);
        }
        request
    }

    async fn send_json<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
        what: &str,
    ) -> Result<T> {
        let response = request
            .send()
            .await
            .map_err(|e| Error::UnityCatalog(format!("{what}: network/CORS: {e}")))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::UnityCatalog(format!("{what}: reading response: {e}")))?;
        if !status.is_success() {
            let detail = serde_json::from_slice::<ErrorEnvelope>(&bytes)
                .ok()
                .and_then(|env| env.error.and_then(|e| e.message).or(env.message))
                .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
            return Err(Error::UnityCatalog(format!(
                "{what}: HTTP {status}: {detail}"
            )));
        }
        serde_json::from_slice(&bytes).map_err(|e| Error::InvalidResponse(format!("{what}: {e}")))
    }

    /// `GET /delta/v1/catalogs/{c}/schemas/{s}/tables/{t}` — table metadata,
    /// commit tail, and latest ratified version.
    pub async fn load_table(
        &self,
        catalog: &str,
        schema: &str,
        table: &str,
    ) -> Result<LoadTableResponse> {
        let url = self
            .base_url
            .join(&format!(
                "delta/v1/catalogs/{}/schemas/{}/tables/{}",
                encode_segment(catalog),
                encode_segment(schema),
                encode_segment(table),
            ))
            .map_err(|e| Error::InvalidUrl(e.to_string()))?;
        self.send_json(self.request(reqwest::Method::GET, url), "loadTable")
            .await
    }

    /// `POST /temporary-table-credentials` with `operation: READ`.
    pub async fn read_table_credentials(&self, table_uuid: &str) -> Result<TemporaryCredential> {
        let url = self
            .base_url
            .join("temporary-table-credentials")
            .map_err(|e| Error::InvalidUrl(e.to_string()))?;
        let body = serde_json::json!({ "tableId": table_uuid, "operation": "READ" });
        self.send_json(
            self.request(reqwest::Method::POST, url).json(&body),
            "temporary-table-credentials",
        )
        .await
    }

    /// `GET /volumes/{name}` — volume metadata, where `full_name` is the dotted
    /// three-level `catalog.schema.volume` name as ONE path segment (dots
    /// preserved, percent-encoded via [`encode_segment`]).
    pub async fn get_volume(&self, full_name: &str) -> Result<VolumeInfo> {
        let url = self
            .base_url
            .join(&format!("volumes/{}", encode_segment(full_name)))
            .map_err(|e| Error::InvalidUrl(e.to_string()))?;
        self.send_json(self.request(reqwest::Method::GET, url), "getVolume")
            .await
    }

    /// `POST /temporary-volume-credentials` with `operation: READ_VOLUME`. Keyed
    /// by the volume's UUID (from [`get_volume`](Self::get_volume)), it returns
    /// the same [`TemporaryCredential`] shape the table path parses.
    pub async fn read_volume_credentials(&self, volume_id: &str) -> Result<TemporaryCredential> {
        let url = self
            .base_url
            .join("temporary-volume-credentials")
            .map_err(|e| Error::InvalidUrl(e.to_string()))?;
        let body = serde_json::json!({ "volumeId": volume_id, "operation": "READ_VOLUME" });
        self.send_json(
            self.request(reqwest::Method::POST, url).json(&body),
            "temporary-volume-credentials",
        )
        .await
    }
}

/// Minimal projection of `VolumeInfo` (the `GetVolume` response) — enough to vend
/// a read credential and resolve the storage location. camelCase with
/// snake_case tolerated, mirroring [`crate::creds::TemporaryCredential`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
    /// The volume's UUID — the `temporary-volume-credentials` request key.
    #[serde(alias = "volume_id")]
    pub volume_id: String,
    /// The volume's cloud storage-location root (e.g. `abfss://…`, `gs://…`).
    #[serde(alias = "storage_location")]
    pub storage_location: String,
    /// The volume type (`MANAGED` / `EXTERNAL`); optional and informational.
    #[serde(default, alias = "volume_type")]
    pub volume_type: Option<String>,
}
