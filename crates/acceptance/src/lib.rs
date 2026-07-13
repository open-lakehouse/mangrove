//! Unity Catalog Acceptance Testing Framework
//!
//! This crate provides a simplified framework for testing Unity Catalog workflows
//! through user journeys written in Rust.

pub mod checks;
pub mod conformance;
pub mod context;
pub mod execution;
pub mod journeys;
pub mod reporting;

pub use context::JourneyContext;

// Re-export commonly used types for convenience
pub use execution::{
    ExecutionMode, ImplementationProfile, ImplementationTag, JourneyConfig, JourneyExecutionResult,
    JourneyExecutor, JourneyFilter, JourneyMetadata, JourneyTier, ResourceTag, UserJourney,
};
pub use execution::{JourneyLogger, PerformanceMetrics, ProgressTracker, cleanup_step};
pub use reporting::{JourneyReporter, ReportingConfig, generate_journeys_summary_table};

/// Result type commonly used throughout the framework
pub type AcceptanceResult<T> = Result<T, AcceptanceError>;

/// Common error types for the acceptance framework
#[derive(Debug, thiserror::Error)]
pub enum AcceptanceError {
    #[error("Journey execution failed: {0}")]
    JourneyExecution(String),

    #[error("Journey validation failed: {0}")]
    JourneyValidation(String),

    #[error("Step execution failed: {step_id}: {message}")]
    StepExecution { step_id: String, message: String },

    #[error("Variable substitution failed: {0}")]
    VariableSubstitution(String),

    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unity Catalog client error: {0}")]
    UnityCatalog(String),

    #[error("Recording error: {0}")]
    Recording(String),
}

impl From<unitycatalog_client::Error> for AcceptanceError {
    fn from(err: unitycatalog_client::Error) -> Self {
        AcceptanceError::UnityCatalog(err.to_string())
    }
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
