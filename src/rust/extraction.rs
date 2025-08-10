//! Crate extraction and example searching

use crate::{Result, EgError, Example, SearchRange};
use flate2::read::GzDecoder;
use regex::Regex;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Handles extraction of examples from .crate files
pub struct CrateExtractor;

impl CrateExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract examples from a cached .crate file
    pub async fn extract_examples_from_file(
        &self,
        crate_path: &Path,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Example>> {
        let file = std::fs::File::open(crate_path)?;
        self.extract_examples_from_reader(file, pattern).await
    }

    /// Download and extract examples from crates.io
    pub async fn extract_examples_from_download(
        &self,
        crate_name: &str,
        version: &str,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Example>> {
        let download_url = format!(
            "https://static.crates.io/crates/{}/{}-{}.crate",
            crate_name, crate_name, version
        );

        let response = reqwest::get(&download_url).await?;
        if !response.status().is_success() {
            return Err(EgError::DownloadError(format!(
                "Failed to download crate: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        self.extract_examples_from_reader(std::io::Cursor::new(bytes), pattern).await
    }

    /// Extract examples from any reader (file or downloaded bytes)
    async fn extract_examples_from_reader<R: Read>(
        &self,
        reader: R,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Example>> {
        let gz_decoder = GzDecoder::new(reader);
        let mut archive = Archive::new(gz_decoder);
        let mut examples = Vec::new();

        for entry_result in archive.entries()
            .map_err(|e| EgError::ExtractionError(format!("Failed to read archive entries: {}", e)))? 
        {
            let mut entry = entry_result
                .map_err(|e| EgError::ExtractionError(format!("Failed to read archive entry: {}", e)))?;
            
            let path = entry.path()
                .map_err(|e| EgError::ExtractionError(format!("Failed to get entry path: {}", e)))?;

            // Check if this is an example file
            if self.is_example_file(&path) {
                let mut content = String::new();
                entry.read_to_string(&mut content)
                    .map_err(|e| EgError::ExtractionError(format!("Failed to read file content: {}", e)))?;

                let search_matches = if let Some(regex) = pattern {
                    self.find_matches(&content, regex)
                } else {
                    Vec::new()
                };

                // Extract just the filename from the full path
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                examples.push(Example::ExampleInMemory {
                    filename,
                    contents: content,
                    search_matches,
                });
            }
        }

        Ok(examples)
    }

    /// Check if a path represents an example file
    fn is_example_file(&self, path: &Path) -> bool {
        // Look for files in examples/ directory
        path.components().any(|c| c.as_os_str() == "examples") &&
        path.extension().map_or(false, |ext| ext == "rs")
    }

    /// Find regex matches in content and convert to SearchRange
    fn find_matches(&self, content: &str, regex: &Regex) -> Vec<SearchRange> {
        let mut matches = Vec::new();
        
        for mat in regex.find_iter(content) {
            let start_pos = mat.start();
            let end_pos = mat.end();
            
            // Calculate line and column positions
            let (start_line, start_col) = self.byte_to_line_col(content, start_pos);
            let (end_line, end_col) = self.byte_to_line_col(content, end_pos);
            
            matches.push(SearchRange {
                byte_start: start_pos as u32,
                line_start: start_line,
                column_start: start_col,
                byte_end: end_pos as u32,
                line_end: end_line,
                column_end: end_col,
            });
        }
        
        matches
    }

    /// Convert byte position to 1-based line and column
    fn byte_to_line_col(&self, content: &str, byte_pos: usize) -> (u32, u32) {
        let mut line = 1;
        let mut col = 1;
        
        for (i, ch) in content.char_indices() {
            if i >= byte_pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }
}
