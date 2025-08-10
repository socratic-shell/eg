//! Cargo cache management

use crate::{Result, EgError};
use std::path::{Path, PathBuf};

/// Manages access to cargo's local cache
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cargo_home = home::cargo_home()
            .map_err(EgError::CargoHomeNotFound)?;
        
        let cache_dir = cargo_home.join("registry").join("cache");
        
        Ok(Self { cache_dir })
    }

    /// Find a cached .crate file for the given crate and version
    pub fn find_cached_crate(&self, crate_name: &str, version: &str) -> Result<Option<PathBuf>> {
        // Standard crates.io cache structure
        let registry_hash_prefix = "github.com-1ecc6299db9ec823";
        let crate_filename = format!("{}-{}.crate", crate_name, version);
        let expected_path = self.cache_dir
            .join(registry_hash_prefix)
            .join(crate_filename);

        if expected_path.exists() {
            Ok(Some(expected_path))
        } else {
            Ok(None)
        }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}
