//! # eg - Example Search Library
//!
//! Programmatic access to library examples and documentation.
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use eg::Eg;
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Find examples in current project's tokio dependency
//!     let result = Eg::rust_crate("tokio").search().await?;
//!     
//!     println!("Found {} examples, {} matched", 
//!              result.total_examples, result.matched_examples);
//!     
//!     Ok(())
//! }
//! ```

use std::ops::Range;
use std::path::PathBuf;

pub mod rust;
pub mod error;

pub use error::{EgError, Result};

/// Main entry point for example searches
pub struct Eg;

impl Eg {
    /// Search for examples in a Rust crate
    pub fn rust_crate(name: &str) -> rust::RustCrateSearch {
        rust::RustCrateSearch::new(name)
    }
}

/// Result of an example search
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The exact version that was searched
    pub version: String,
    /// Total number of example files found
    pub total_examples: usize,
    /// Number of examples that matched the search pattern
    pub matched_examples: usize,
    /// The actual example files and their matches
    pub examples: Vec<Example>,
}

/// An example file with optional search matches
#[derive(Debug, Clone)]
pub enum Example {
    /// Example found on local disk
    ExampleOnDisk {
        /// Local file path
        path: PathBuf,
        /// Locations where search pattern matched
        search_matches: Vec<SearchRange>,
    },
    /// Example downloaded and held in memory
    ExampleInMemory {
        /// Original path in examples/ directory
        filename: String,
        /// File contents
        contents: String,
        /// Locations where search pattern matched
        search_matches: Vec<SearchRange>,
    },
}

/// Precise location of a search match within a file
#[derive(Debug, Clone)]
pub struct SearchRange {
    /// 0-based byte that is the start of the match
    pub byte_start: u32,
    /// 1-based line where the search started
    pub line_start: u32,
    /// 1-based column marking the start of the match
    pub column_start: u32,
    /// 0-based byte that is the end of the match (exclusive)
    pub byte_end: u32,
    /// 1-based line where the search ended
    pub line_end: u32,
    /// 1-based column marking the end of the match (exclusive)
    pub column_end: u32,
}
