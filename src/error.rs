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
    /// Crate not found
    #[error("Crate '{0}' not found")]
    CrateNotFound(String),
    /// No matching versions found
    #[error("No versions of '{crate_name}' match constraint '{constraint}'")]
    NoMatchingVersions { crate_name: String, constraint: String },
    /// No repository URL found
    #[error("No repository URL found for crate '{0}'")]
    NoRepositoryUrl(String),
    /// Invalid GitHub URL format
    #[error("Invalid GitHub URL format: {0}")]
    InvalidGitHubUrl(String),
    /// Base64 decode error
    #[error("Failed to decode base64 content: {0}")]
    Base64Error(#[from] base64::DecodeError),
    /// UTF-8 conversion error
    #[error("Invalid UTF-8 content: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    /// Other error
    #[error("Error: {0}")]
    Other(String),
}
