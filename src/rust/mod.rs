//! Rust-specific example searching functionality

use crate::{Result, SearchResult, Example, SearchRange};
use regex::Regex;

mod version;
mod cache;
mod extraction;
mod github;

pub use version::VersionResolver;
pub use cache::CacheManager;
pub use extraction::CrateExtractor;
pub use github::GitHubFallback;

/// Builder for searching Rust crate examples
pub struct RustCrateSearch {
    crate_name: String,
    version_spec: Option<String>,
    pattern: Option<Regex>,
}

impl RustCrateSearch {
    /// Create a new search for the given crate name
    pub fn new(name: &str) -> Self {
        Self {
            crate_name: name.to_string(),
            version_spec: None,
            pattern: None,
        }
    }

    /// Specify a version constraint (e.g., "^1.0", "=1.2.3")
    pub fn version(mut self, version: &str) -> Self {
        self.version_spec = Some(version.to_string());
        self
    }

    /// Specify a regex pattern to search for within examples
    pub fn pattern(mut self, pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| crate::EgError::Other(format!("Invalid regex pattern: {}", e)))?;
        self.pattern = Some(regex);
        Ok(self)
    }

    /// Execute the search
    pub async fn search(self) -> Result<SearchResult> {
        // 1. Resolve version
        let resolver = VersionResolver::new();
        let version = resolver.resolve_version(&self.crate_name, self.version_spec.as_deref()).await?;

        // 2. Try to find examples in crate source
        let cache_manager = CacheManager::new()?;
        let extractor = CrateExtractor::new();
        
        let examples = if let Some(cached_path) = cache_manager.find_cached_crate(&self.crate_name, &version)? {
            // Extract from cached crate
            extractor.extract_examples_from_file(&cached_path, self.pattern.as_ref()).await?
        } else {
            // Download and extract
            extractor.extract_examples_from_download(&self.crate_name, &version, self.pattern.as_ref()).await?
        };

        // 3. If no examples found, try GitHub fallback
        let final_examples = if examples.is_empty() {
            let github = GitHubFallback::new();
            github.search_examples(&self.crate_name, &version, self.pattern.as_ref()).await?
        } else {
            examples
        };

        // 4. Build result
        let total_examples = final_examples.len();
        let matched_examples = if self.pattern.is_some() {
            final_examples.iter().filter(|e| !e.search_matches().is_empty()).count()
        } else {
            total_examples
        };

        Ok(SearchResult {
            version,
            total_examples,
            matched_examples,
            examples: final_examples,
        })
    }
}

impl Example {
    /// Get search matches for this example
    pub fn search_matches(&self) -> &[SearchRange] {
        match self {
            Example::ExampleOnDisk { search_matches, .. } => search_matches,
            Example::ExampleInMemory { search_matches, .. } => search_matches,
        }
    }
}
