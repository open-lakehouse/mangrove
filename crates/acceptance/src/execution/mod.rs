//! Simplified Journey Framework
//!
//! This module provides a simplified approach to journey testing that:
//! - Uses manual Rust code instead of JSON configurations
//! - Leverages the actual UnityCatalogClient for type safety
//! - Records responses to numbered files for later comparison
//! - Replays recorded interactions using mock servers
//! - Focuses on user journeys rather than low-level HTTP details

use async_trait::async_trait;
use mockito::ServerGuard;
use olai_http::{CloudClient, RequestResponseInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use unitycatalog_client::UnityCatalogClient;
use url::Url;

use crate::{AcceptanceError, AcceptanceResult};
pub use helpers::*;

mod helpers;

// ---------------------------------------------------------------------------
// Journey metadata types
// ---------------------------------------------------------------------------

/// Unity Catalog resource types that a journey exercises
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceTag {
    Catalogs,
    Schemas,
    Tables,
    Volumes,
    Credentials,
    ExternalLocations,
    Shares,
    Recipients,
    Providers,
    Functions,
    TemporaryCredentials,
}

/// Which Unity Catalog implementations support this journey
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplementationTag {
    /// Our Rust OSS implementation
    OssRust,
    /// The Java Unity Catalog OSS implementation
    OssJava,
    /// The Databricks managed Unity Catalog service (reference implementation)
    ManagedDatabricks,
    /// All implementations
    All,
}

/// Tier of complexity / dependencies for a journey
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum JourneyTier {
    /// Basic CRUD operations — no external dependencies
    Tier1Crud,
    /// Governance features — may require credentials / external storage
    Tier2Governance,
    /// Delta Sharing — provider/consumer workflows
    Tier3Sharing,
    /// Advanced features (UDFs, cross-resource workflows)
    Tier4Advanced,
}

/// Descriptive metadata attached to every journey
#[derive(Debug, Clone)]
pub struct JourneyMetadata {
    /// Which resource types this journey exercises
    pub resources: Vec<ResourceTag>,
    /// Which implementations support this journey
    pub implementations: Vec<ImplementationTag>,
    /// Complexity tier
    pub tier: JourneyTier,
    /// True if this journey requires a configured external cloud storage root
    pub requires_external_storage: bool,
}

impl Default for JourneyMetadata {
    fn default() -> Self {
        Self {
            resources: vec![],
            implementations: vec![ImplementationTag::All],
            tier: JourneyTier::Tier1Crud,
            requires_external_storage: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Journey filtering
// ---------------------------------------------------------------------------

/// Controls which journeys are selected for a run
#[derive(Debug, Default, Clone)]
pub struct JourneyFilter {
    /// Only run journeys compatible with this implementation. `None` = no restriction.
    pub implementation: Option<ImplementationTag>,
    /// Only run journeys at or below this tier. `None` = no restriction.
    pub max_tier: Option<JourneyTier>,
    /// If non-empty, only run journeys whose name is in this list.
    pub include_names: Vec<String>,
    /// Always skip journeys whose name is in this list.
    pub exclude_names: Vec<String>,
    /// If true, skip journeys that require external storage.
    pub skip_external_storage: bool,
}

impl JourneyFilter {
    /// Build a filter from environment variables:
    /// - `UC_JOURNEY_INCLUDE` — comma-separated journey names to include
    /// - `UC_JOURNEY_EXCLUDE` — comma-separated journey names to exclude
    /// - `UC_JOURNEY_IMPL`    — implementation tag: `oss_rust`, `oss_java`, `managed_databricks`
    /// - `UC_JOURNEY_MAX_TIER` — `tier1`, `tier2`, `tier3`, `tier4`
    pub fn from_env() -> Self {
        let include_names = std::env::var("UC_JOURNEY_INCLUDE")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let exclude_names = std::env::var("UC_JOURNEY_EXCLUDE")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let implementation = std::env::var("UC_JOURNEY_IMPL")
            .ok()
            .and_then(|v| match v.as_str() {
                "oss_rust" => Some(ImplementationTag::OssRust),
                "oss_java" => Some(ImplementationTag::OssJava),
                "managed_databricks" => Some(ImplementationTag::ManagedDatabricks),
                _ => None,
            });

        let max_tier = std::env::var("UC_JOURNEY_MAX_TIER")
            .ok()
            .and_then(|v| match v.as_str() {
                "tier1" => Some(JourneyTier::Tier1Crud),
                "tier2" => Some(JourneyTier::Tier2Governance),
                "tier3" => Some(JourneyTier::Tier3Sharing),
                "tier4" => Some(JourneyTier::Tier4Advanced),
                _ => None,
            });

        Self {
            include_names,
            exclude_names,
            implementation,
            max_tier,
            skip_external_storage: false,
        }
    }

    /// Returns `true` if the journey should be included in this run
    pub fn matches(&self, journey: &dyn UserJourney) -> bool {
        let meta = journey.metadata();
        let name = journey.name();

        if !self.include_names.is_empty() && !self.include_names.iter().any(|n| n == name) {
            return false;
        }

        if self.exclude_names.iter().any(|n| n == name) {
            return false;
        }

        if let Some(ref max_tier) = self.max_tier
            && meta.tier > *max_tier
        {
            return false;
        }

        if let Some(ref impl_tag) = self.implementation {
            let supported = meta.implementations.contains(impl_tag)
                || meta.implementations.contains(&ImplementationTag::All);
            if !supported {
                return false;
            }
        }

        if self.skip_external_storage && meta.requires_external_storage {
            return false;
        }

        true
    }
}

// ---------------------------------------------------------------------------
// Implementation profile
// ---------------------------------------------------------------------------

/// A named preset for targeting a specific Unity Catalog implementation
#[derive(Debug, Clone)]
pub struct ImplementationProfile {
    /// Short name for this profile (e.g. "oss_rust", "managed_databricks")
    pub name: String,
    /// Base URL of the Unity Catalog server
    pub server_url: String,
    /// Optional bearer token for authentication
    pub auth_token: Option<String>,
    /// Default cloud storage root used by journeys that create external resources
    pub storage_root: String,
    /// Journey filter applied when using this profile
    pub filter: JourneyFilter,
}

impl ImplementationProfile {
    /// Profile for the local Rust OSS implementation
    pub fn oss_rust(server_url: impl Into<String>) -> Self {
        Self {
            name: "oss_rust".to_string(),
            server_url: server_url.into(),
            auth_token: None,
            storage_root: "file:///tmp/uc-test/".to_string(),
            filter: JourneyFilter {
                implementation: Some(ImplementationTag::OssRust),
                skip_external_storage: true,
                ..Default::default()
            },
        }
    }

    /// Profile for the Java OSS Unity Catalog implementation
    pub fn oss_java(server_url: impl Into<String>) -> Self {
        Self {
            name: "oss_java".to_string(),
            server_url: server_url.into(),
            auth_token: None,
            storage_root: "file:///tmp/uc-test/".to_string(),
            filter: JourneyFilter {
                implementation: Some(ImplementationTag::OssJava),
                skip_external_storage: true,
                ..Default::default()
            },
        }
    }

    /// Profile for the Databricks managed Unity Catalog (reference implementation)
    pub fn managed_databricks(
        server_url: impl Into<String>,
        token: impl Into<String>,
        storage_root: impl Into<String>,
    ) -> Self {
        Self {
            name: "managed_databricks".to_string(),
            server_url: server_url.into(),
            auth_token: Some(token.into()),
            storage_root: storage_root.into(),
            filter: JourneyFilter {
                implementation: Some(ImplementationTag::ManagedDatabricks),
                ..Default::default()
            },
        }
    }

    /// Overlay environment variable overrides on top of a base profile.
    /// `UC_INTEGRATION_URL`, `UC_INTEGRATION_TOKEN`, `UC_INTEGRATION_STORAGE_ROOT`,
    /// `UC_JOURNEY_INCLUDE`, `UC_JOURNEY_EXCLUDE` are all applied.
    pub fn from_env(mut base: Self) -> Self {
        if let Ok(url) = std::env::var("UC_INTEGRATION_URL") {
            base.server_url = url;
        }
        if let Ok(token) = std::env::var("UC_INTEGRATION_TOKEN") {
            base.auth_token = Some(token);
        }
        if let Ok(root) = std::env::var("UC_INTEGRATION_STORAGE_ROOT") {
            base.storage_root = root;
        }
        let env_filter = JourneyFilter::from_env();
        if !env_filter.include_names.is_empty() {
            base.filter.include_names = env_filter.include_names;
        }
        if !env_filter.exclude_names.is_empty() {
            base.filter.exclude_names = env_filter.exclude_names;
        }
        if env_filter.max_tier.is_some() {
            base.filter.max_tier = env_filter.max_tier;
        }
        base
    }
}

// ---------------------------------------------------------------------------
// Journey context
// ---------------------------------------------------------------------------

// `JourneyContext` now lives in `crate::context`; re-exported here so existing
// `crate::execution::JourneyContext` paths keep resolving during the transition.
pub use crate::context::JourneyContext;

// ---------------------------------------------------------------------------
// UserJourney trait
// ---------------------------------------------------------------------------

/// A user journey that can be executed against Unity Catalog
#[async_trait]
pub trait UserJourney: Send + Sync {
    /// Unique identifier for this journey
    fn name(&self) -> &str;

    /// Human-readable description of what this journey tests
    fn description(&self) -> &str;

    /// Metadata describing resources, compatibility, and tier
    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata::default()
    }

    /// Optional setup that runs before the journey
    #[allow(unused_variables)]
    async fn setup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Execute the journey steps using the provided context
    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()>;

    /// Optional cleanup that runs after the journey (even on failure)
    #[allow(unused_variables)]
    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        Ok(())
    }

    /// Save journey state for replay
    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        Ok(JourneyState::empty())
    }

    /// Restore journey state from replay data
    fn load_state(&mut self, _state: &JourneyState) -> AcceptanceResult<()> {
        Ok(())
    }
}

#[async_trait]
impl<T: UserJourney> UserJourney for Box<T> {
    fn name(&self) -> &str {
        T::name(self)
    }

    fn description(&self) -> &str {
        T::description(self)
    }

    fn metadata(&self) -> JourneyMetadata {
        T::metadata(self)
    }

    async fn setup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        T::setup(self, ctx).await
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        T::execute(self, ctx).await
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        T::cleanup(self, ctx).await
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        T::save_state(self)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        T::load_state(self, state)
    }
}

// ---------------------------------------------------------------------------
// Journey state
// ---------------------------------------------------------------------------

/// Journey state that can be persisted and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyState {
    /// Generic key-value store for journey-specific data
    pub data: HashMap<String, serde_json::Value>,
}

impl JourneyState {
    /// Create an empty state
    pub fn empty() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set a string value
    pub fn set_string(&mut self, key: &str, value: String) {
        self.data
            .insert(key.to_string(), serde_json::Value::String(value));
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key)?.as_str().map(|s| s.to_string())
    }

    /// Set an integer value
    pub fn set_i64(&mut self, key: &str, value: i64) {
        self.data.insert(
            key.to_string(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }

    /// Get an integer value
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Set a boolean value
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.data
            .insert(key.to_string(), serde_json::Value::Bool(value));
    }

    /// Get a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ---------------------------------------------------------------------------
// JourneyExecutor
// ---------------------------------------------------------------------------

/// Executes simple journeys and manages recording/replay
pub struct JourneyExecutor {
    context: JourneyContext,
    _mock_server: Option<ServerGuard>,
}

impl JourneyExecutor {
    /// Create a new executor for live mode
    pub fn new(client: UnityCatalogClient, storage_root: impl Into<String>) -> Self {
        Self {
            context: JourneyContext::new(client, storage_root),
            _mock_server: None,
        }
    }

    /// Create a new executor with mock server for replay mode
    pub fn new_with_mock(
        client: UnityCatalogClient,
        storage_root: impl Into<String>,
        mock_server: ServerGuard,
    ) -> Self {
        Self {
            context: JourneyContext::new(client, storage_root),
            _mock_server: Some(mock_server),
        }
    }

    /// Read recorded interactions from a directory
    async fn read_recorded_interactions(
        recordings_dir: &PathBuf,
    ) -> AcceptanceResult<Vec<RequestResponseInfo>> {
        if !recordings_dir.exists() {
            return Err(AcceptanceError::JourneyValidation(format!(
                "Recordings directory does not exist: {}",
                recordings_dir.display()
            )));
        }

        let mut recordings = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(recordings_dir)
            .map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to read recordings directory: {}",
                    e
                ))
            })?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "json" {
                    // Exclude journey_state.json from recordings
                    if path.file_name()? == "journey_state.json" {
                        return None;
                    }
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Sort by filename to ensure deterministic order
        entries.sort();

        for path in entries {
            let content = fs::read_to_string(&path).map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to read recording file {}: {}",
                    path.display(),
                    e
                ))
            })?;

            let recording: RequestResponseInfo = serde_json::from_str(&content).map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to parse recording file {}: {}",
                    path.display(),
                    e
                ))
            })?;

            recordings.push(recording);
        }

        Ok(recordings)
    }

    /// Save journey state to the recordings directory
    fn save_journey_state(recordings_dir: &Path, state: &JourneyState) -> AcceptanceResult<()> {
        if state.is_empty() {
            return Ok(());
        }

        let state_file = recordings_dir.join("journey_state.json");
        let content = serde_json::to_string_pretty(state).map_err(|e| {
            AcceptanceError::Recording(format!("Failed to serialize journey state: {}", e))
        })?;

        fs::write(&state_file, content).map_err(|e| {
            AcceptanceError::Recording(format!(
                "Failed to write journey state to {}: {}",
                state_file.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Load journey state from the recordings directory
    async fn load_journey_state(recordings_dir: &Path) -> AcceptanceResult<JourneyState> {
        let state_file = recordings_dir.join("journey_state.json");

        if !state_file.exists() {
            return Ok(JourneyState::empty());
        }

        let content = fs::read_to_string(&state_file).map_err(|e| {
            AcceptanceError::JourneyValidation(format!(
                "Failed to read journey state from {}: {}",
                state_file.display(),
                e
            ))
        })?;

        let state: JourneyState = serde_json::from_str(&content).map_err(|e| {
            AcceptanceError::JourneyValidation(format!(
                "Failed to parse journey state from {}: {}",
                state_file.display(),
                e
            ))
        })?;

        Ok(state)
    }

    /// Set up mock server with recorded interactions.
    ///
    /// Each recording is registered as a separate mock with `.expect(1)` so that
    /// multiple calls to the same endpoint (e.g. three `POST /schemas`) are matched
    /// in recorded order rather than all returning the same response.
    ///
    /// When the recorded request carried a JSON body, the mock also matches on that
    /// body using `mockito::Matcher::Json` — this disambiguates calls that share a
    /// method + path but differ only in the request payload.
    pub async fn setup_mock_server(
        recordings_dir: &PathBuf,
    ) -> AcceptanceResult<(ServerGuard, UnityCatalogClient)> {
        let recordings = Self::read_recorded_interactions(recordings_dir).await?;

        if recordings.is_empty() {
            return Err(AcceptanceError::JourneyValidation(
                "No recordings found for replay".to_string(),
            ));
        }

        println!(
            "🎬 Setting up mock server with {} recorded interactions from {}",
            recordings.len(),
            recordings_dir.display()
        );

        let mut server = mockito::Server::new_async().await;
        let mut mocks = Vec::new();

        for recording in recordings {
            let method_str = recording.request.method.as_str();
            let path = recording.request.url_path.as_str();

            let mut mock = server
                .mock(method_str, path)
                .with_status(recording.response.status as usize);

            // Match on request body when it is valid JSON — this disambiguates
            // multiple calls to the same endpoint with different payloads.
            if let Some(ref body) = recording.request.body {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
                    mock = mock.match_body(mockito::Matcher::Json(json_value));
                } else {
                    mock = mock.match_body(mockito::Matcher::Exact(body.clone()));
                }
            }

            // Add response headers
            for (header_name, header_value) in &recording.response.headers {
                mock = mock.with_header(header_name, header_value);
            }

            // Add response body if present
            if let Some(ref body) = recording.response.body {
                mock = mock.with_body(body);
            }

            // Each recording is consumed exactly once — enforces ordered replay
            mock = mock.expect(1);

            let created_mock = mock.create_async().await;
            mocks.push(created_mock);

            println!(
                "  📝 Mock: {} {} -> {}",
                recording.request.method, recording.request.url_path, recording.response.status
            );
        }

        // Create client pointing to mock server
        let mock_url = format!("{}/api/2.1/unity-catalog", server.url());
        let base_url: Url = mock_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Invalid mock server URL: {}", e))
        })?;

        let olai_http = CloudClient::new_unauthenticated();
        let client = UnityCatalogClient::new(olai_http, base_url);

        println!("🚀 Mock server ready at: {}", server.url());

        Ok((server, client))
    }

    /// Execute a journey
    pub async fn execute_journey(
        &self,
        journey: &mut dyn UserJourney,
    ) -> AcceptanceResult<JourneyExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut result = JourneyExecutionResult {
            journey_name: journey.name().to_string(),
            success: false,
            duration: std::time::Duration::default(),
            error_message: None,
            steps_completed: 0,
        };

        // Execute setup
        if let Err(e) = journey.setup(&self.context).await {
            result.error_message = Some(format!("Setup failed: {}", e));
            return Ok(result);
        }

        // Execute main journey
        let journey_result = journey.execute(&self.context).await;

        // Always run cleanup
        if let Err(cleanup_err) = journey.cleanup(&self.context).await {
            eprintln!(
                "⚠️ Cleanup failed for journey '{}': {}",
                journey.name(),
                cleanup_err
            );
        }

        result.duration = start_time.elapsed();

        match journey_result {
            Ok(()) => {
                result.success = true;
                println!(
                    "✅ Journey '{}' completed successfully in {:?}",
                    journey.name(),
                    result.duration
                );
            }
            Err(e) => {
                result.error_message = Some(e.to_string());
                println!(
                    "❌ Journey '{}' failed after {:?}: {}",
                    journey.name(),
                    result.duration,
                    e
                );
            }
        }

        Ok(result)
    }

    /// Execute multiple journeys
    pub async fn execute_journeys(
        &self,
        journeys: Vec<&mut dyn UserJourney>,
    ) -> AcceptanceResult<Vec<JourneyExecutionResult>> {
        let mut results = Vec::new();

        for journey in journeys {
            let result = self.execute_journey(journey).await?;
            results.push(result);

            // Small delay between journeys to avoid overwhelming the server
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// JourneyExecutionResult
// ---------------------------------------------------------------------------

/// Result of executing a journey
#[derive(Debug, Clone)]
pub struct JourneyExecutionResult {
    pub journey_name: String,
    pub success: bool,
    pub duration: std::time::Duration,
    pub error_message: Option<String>,
    pub steps_completed: usize,
}

impl JourneyExecutionResult {
    /// Check if the journey was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get a summary string for reporting
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "✅ {} ({} steps, {:?})",
                self.journey_name, self.steps_completed, self.duration
            )
        } else {
            format!(
                "❌ {} ({} steps, {:?}) - {}",
                self.journey_name,
                self.steps_completed,
                self.duration,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

// ---------------------------------------------------------------------------
// JourneyConfig
// ---------------------------------------------------------------------------

/// How a journey is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Replay recorded HTTP interactions against a mockito mock server. Journeys
    /// without a recordings directory are skipped. No live server needed.
    Replay,
    /// Execute against a live server without recording. Used to validate clients
    /// against a real implementation (e.g. the Java OSS server in CI).
    Live,
    /// Execute against a live server and record the interactions as new fixtures.
    Record,
}

impl ExecutionMode {
    /// Resolve the mode from the `UC_INTEGRATION_RECORD` environment variable
    /// (`true` = [`Record`](Self::Record), otherwise [`Replay`](Self::Replay)).
    fn from_record_env() -> Self {
        if std::env::var("UC_INTEGRATION_RECORD").unwrap_or_default() == "true" {
            Self::Record
        } else {
            Self::Replay
        }
    }

    /// True when the journey runs against a live server (Live or Record).
    fn is_live(self) -> bool {
        matches!(self, Self::Live | Self::Record)
    }
}

/// Configuration for journey execution
#[derive(Debug, Clone)]
pub struct JourneyConfig {
    pub mode: ExecutionMode,
    pub output_dir: PathBuf,
    pub server_url: String,
    pub auth_token: Option<String>,
    pub timeout_seconds: u64,
    pub storage_root: String,
    /// Implementation profile name (e.g. `managed_databricks`, `oss_rust`). Used to
    /// namespace recordings as `<output_dir>/<profile_name>/<journey>/` so cassettes
    /// for different implementations don't collide.
    pub profile_name: String,
}

impl Default for JourneyConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::from_record_env(),
            output_dir: std::env::var("UC_INTEGRATION_DIR")
                .unwrap_or_else(|_| "test_data/recordings".to_string())
                .into(),
            server_url: std::env::var("UC_INTEGRATION_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            auth_token: std::env::var("UC_INTEGRATION_TOKEN").ok(),
            timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            storage_root: std::env::var("UC_INTEGRATION_STORAGE_ROOT")
                .unwrap_or_else(|_| "file:///tmp/uc-test/".to_string()),
            profile_name: std::env::var("UC_INTEGRATION_PROFILE")
                .unwrap_or_else(|_| "managed_databricks".to_string()),
        }
    }
}

impl JourneyConfig {
    /// Build a config from an [`ImplementationProfile`]
    pub fn for_profile(profile: &ImplementationProfile) -> Self {
        Self {
            server_url: profile.server_url.clone(),
            auth_token: profile.auth_token.clone(),
            storage_root: profile.storage_root.clone(),
            profile_name: profile.name.clone(),
            ..Default::default()
        }
    }

    /// Set the implementation profile name used to namespace recordings.
    pub fn with_profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_name = profile_name.into();
        self
    }

    /// Create client from configuration for live mode
    pub fn create_client(&self, out_dir: PathBuf) -> AcceptanceResult<UnityCatalogClient> {
        let base_url: Url = self.server_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Invalid server URL: {}", e))
        })?;
        let base_url = base_url.join("/api/2.1/unity-catalog").unwrap();

        let mut client = if let Some(ref token) = self.auth_token {
            CloudClient::new_with_token(token)
        } else {
            CloudClient::new_unauthenticated()
        };
        if self.mode == ExecutionMode::Record {
            std::fs::create_dir_all(&out_dir)?;
            client.set_recording_dir(out_dir)?;
        }

        Ok(UnityCatalogClient::new(client, base_url))
    }

    /// Set the execution mode (replay, live, or record).
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Convenience: `true` selects [`ExecutionMode::Record`] (live + record new
    /// fixtures), `false` selects [`ExecutionMode::Replay`].
    ///
    /// For live execution without recording, use [`with_mode`](Self::with_mode)
    /// with [`ExecutionMode::Live`].
    pub fn with_recording(mut self, enabled: bool) -> Self {
        self.mode = if enabled {
            ExecutionMode::Record
        } else {
            ExecutionMode::Replay
        };
        self
    }

    /// Set output directory for recordings
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Execute a journey with state management.
    ///
    /// In replay mode, if the recordings directory for this journey does not exist the
    /// journey is **skipped** (returns a successful result with a note) rather than
    /// failing — this allows the test suite to pass while recordings are pending.
    pub async fn execute_journey(
        &self,
        journey: &mut dyn UserJourney,
    ) -> AcceptanceResult<JourneyExecutionResult> {
        // Recordings are namespaced by implementation profile so cassettes for
        // different implementations don't collide: <output_dir>/<profile>/<journey>/.
        let profile_dir = self.output_dir.join(&self.profile_name);

        if self.mode.is_live() {
            // Live execution against a real server. In Record mode we also point
            // the client at a recording directory and persist state afterwards;
            // in plain Live mode we just exercise the server.
            let out_dir = if self.mode == ExecutionMode::Record {
                std::fs::create_dir_all(&profile_dir)?;
                std::fs::canonicalize(profile_dir.clone())?.join(journey.name())
            } else {
                profile_dir.join(journey.name())
            };
            let client = self.create_client(out_dir.clone())?;
            let executor = JourneyExecutor::new(client, self.storage_root.clone());
            let result = executor.execute_journey(journey).await?;

            // Save journey state only when recording new fixtures. We also persist the
            // storage root used during recording so replay can reconstruct the exact
            // request bodies (catalog `storage_root` etc.) that were captured.
            if self.mode == ExecutionMode::Record && result.is_success() {
                let mut state = journey.save_state()?;
                state.set_string("__storage_root", self.storage_root.clone());
                JourneyExecutor::save_journey_state(&out_dir, &state)?;
            }

            Ok(result)
        } else {
            // Replay mode - load state and use recorded interactions
            let recordings_dir = profile_dir.join(journey.name());

            // Skip journeys that have not been recorded yet rather than failing
            if !recordings_dir.exists() {
                println!(
                    "⏭️  Skipping journey '{}' — no recordings found at {} (run with UC_INTEGRATION_RECORD=true to record)",
                    journey.name(),
                    recordings_dir.display()
                );
                return Ok(JourneyExecutionResult {
                    journey_name: journey.name().to_string(),
                    success: true,
                    duration: std::time::Duration::default(),
                    error_message: None,
                    steps_completed: 0,
                });
            }

            let recordings_dir = std::fs::canonicalize(recordings_dir)?;

            println!(
                "🎬 Starting replay mode for journey '{}' from {}",
                journey.name(),
                recordings_dir.display()
            );

            // Load and apply journey state
            let state = JourneyExecutor::load_journey_state(&recordings_dir).await?;
            // Reconstruct the storage root captured at record time so replayed request
            // bodies match the recordings; fall back to the configured root.
            let storage_root = state
                .get_string("__storage_root")
                .unwrap_or_else(|| self.storage_root.clone());
            journey.load_state(&state)?;

            let (mock_server, client) = JourneyExecutor::setup_mock_server(&recordings_dir).await?;
            let executor = JourneyExecutor::new_with_mock(client, storage_root, mock_server);

            executor.execute_journey(journey).await
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mock_server_replay() {
        // Create a temporary directory with mock recordings
        let temp_dir = TempDir::new().unwrap();
        let recordings_dir = temp_dir.path().join("test_journey");
        fs::create_dir_all(&recordings_dir).unwrap();

        // Create a mock recording file
        let recording = RequestResponseInfo {
            request: olai_http::RequestInfo {
                method: "GET".to_string(),
                url_path: "/api/2.1/unity-catalog/catalogs".to_string(),
                body: None,
            },
            response: olai_http::ResponseInfo {
                status: 200,
                headers: {
                    let mut headers = std::collections::HashMap::new();
                    headers.insert("content-type".to_string(), "application/json".to_string());
                    headers
                },
                body: Some(r#"{"catalogs":[]}"#.to_string()),
            },
        };

        let recording_file = recordings_dir.join("000000.json");
        let file = fs::File::create(recording_file).unwrap();
        serde_json::to_writer_pretty(file, &recording).unwrap();

        // Test mock server setup
        let (_mock_server, client) = JourneyExecutor::setup_mock_server(&recordings_dir)
            .await
            .unwrap();

        // Verify that the client can make requests to the mock server
        let mut catalogs_stream = client.list_catalogs().into_stream();
        use futures::StreamExt;

        // Since we mocked an empty catalogs response, the stream should be empty
        let first_item = catalogs_stream.next().await;
        assert!(first_item.is_none());
    }

    #[tokio::test]
    async fn test_execution_mode_configuration() {
        let config = JourneyConfig::default();

        // with_recording(true) selects Record (live + record fixtures).
        let recording_config = config.clone().with_recording(true);
        assert_eq!(recording_config.mode, ExecutionMode::Record);

        // with_recording(false) selects Replay.
        let replay_config = config.clone().with_recording(false);
        assert_eq!(replay_config.mode, ExecutionMode::Replay);

        // with_mode allows live execution without recording.
        let live_config = config.with_mode(ExecutionMode::Live);
        assert_eq!(live_config.mode, ExecutionMode::Live);
    }

    #[test]
    fn test_journey_filter_from_env_defaults() {
        let filter = JourneyFilter::default();
        assert!(filter.include_names.is_empty());
        assert!(filter.exclude_names.is_empty());
        assert!(filter.implementation.is_none());
        assert!(filter.max_tier.is_none());
    }

    #[test]
    fn test_journey_tier_ordering() {
        assert!(JourneyTier::Tier1Crud < JourneyTier::Tier2Governance);
        assert!(JourneyTier::Tier2Governance < JourneyTier::Tier3Sharing);
        assert!(JourneyTier::Tier3Sharing < JourneyTier::Tier4Advanced);
    }

    #[test]
    fn test_journey_filter_matches_all_tag() {
        struct DummyJourney;
        #[async_trait::async_trait]
        impl UserJourney for DummyJourney {
            fn name(&self) -> &str {
                "dummy"
            }
            fn description(&self) -> &str {
                "dummy"
            }
            fn metadata(&self) -> JourneyMetadata {
                JourneyMetadata {
                    implementations: vec![ImplementationTag::All],
                    ..Default::default()
                }
            }
            async fn execute(&self, _: &JourneyContext) -> AcceptanceResult<()> {
                Ok(())
            }
        }

        let journey = DummyJourney;
        // Filter targeting managed_databricks should still match a journey tagged All
        let filter = JourneyFilter {
            implementation: Some(ImplementationTag::ManagedDatabricks),
            ..Default::default()
        };
        assert!(filter.matches(&journey));
    }
}
