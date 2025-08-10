//! Error types for the eg library

use thiserror::Error;

/// Result type alias for eg operations
pub type Result<T> = std::result::Result<T, EgError>;

/// Errors that can occur during example searching
#[derive(Debug, Error)]
pub enum EgError {
    /// Failed to parse or access project metadata
    #[error("Project error: {0}")]
    ProjectError(#[from] cargo_metadata::Error),
    /// Failed to resolve version constraints
    #[error("Version error: {0}")]
    VersionError(#[from] semver::Error),
    /// Could not determine CARGO_HOME directory
    #[error("Could not determine CARGO_HOME directory")]
    CargoHomeNotFound(#[source] std::io::Error),
    /// Failed to access cargo cache
    #[error("Cache error: {0}")]
    CacheError(String),
    /// Failed to download crate from registry
    #[error("Download error: {0}")]
    DownloadError(#[from] reqwest::Error),
    /// Failed to extract or process crate archive
    #[error("Extraction error: {0}")]
    ExtractionError(String),
    /// Failed to access GitHub repository
    #[error("GitHub error: {0}")]
    GitHubError(#[from] octocrab::Error),
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    /// Other error
    #[error("Error: {0}")]
    Other(String),
}
