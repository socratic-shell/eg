//! GitHub repository fallback for finding examples

use crate::{Result, EgError, Example, SearchRange};
use regex::Regex;
use std::env;

/// Handles GitHub repository fallback when crate sources lack examples
pub struct GitHubFallback;

impl GitHubFallback {
    pub fn new() -> Self {
        Self
    }

    /// Search for examples in GitHub repository
    pub async fn search_examples(
        &self,
        crate_name: &str,
        version: &str,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Example>> {
        // Get repository URL from crate metadata
        let repo_url = self.get_repository_url(crate_name).await?;
        
        // Check if it's a GitHub repository
        if !self.is_github_url(&repo_url) {
            return Ok(Vec::new()); // Skip non-GitHub repositories for now
        }

        // Parse owner/repo from URL
        let (owner, repo) = self.parse_github_url(&repo_url)?;

        // Search examples directory
        self.search_github_examples(&owner, &repo, pattern).await
    }

    /// Get repository URL from crates.io metadata
    async fn get_repository_url(&self, crate_name: &str) -> Result<String> {
        let client = crates_io_api::SyncClient::new(
            "eg-library (https://github.com/socratic-shell/eg)",
            std::time::Duration::from_millis(1000),
        ).map_err(|e| EgError::Other(e.to_string()))?;

        let crate_info = client.get_crate(crate_name)
            .map_err(|e| EgError::DownloadError(format!("Failed to get crate info: {}", e)))?;

        crate_info.crate_data.repository
            .ok_or_else(|| EgError::GitHubError(format!("No repository URL found for crate '{}'", crate_name)))
    }

    /// Check if URL is a GitHub repository
    fn is_github_url(&self, url: &str) -> bool {
        url.contains("github.com")
    }

    /// Parse GitHub URL to extract owner and repository name
    fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        // Handle various GitHub URL formats:
        // https://github.com/owner/repo
        // https://github.com/owner/repo.git
        // git://github.com/owner/repo.git
        
        let url = url.trim_end_matches(".git");
        let parts: Vec<&str> = url.split('/').collect();
        
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2].to_string();
            let repo = parts[parts.len() - 1].to_string();
            Ok((owner, repo))
        } else {
            Err(EgError::GitHubError(format!("Invalid GitHub URL format: {}", url)))
        }
    }

    /// Search examples directory in GitHub repository
    async fn search_github_examples(
        &self,
        owner: &str,
        repo: &str,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Example>> {
        // Try to get GitHub token from environment
        let token = env::var("GITHUB_TOKEN").ok();
        
        let octocrab = if let Some(token) = token {
            octocrab::Octocrab::builder()
                .personal_token(token)
                .build()?
        } else {
            // Use without authentication (lower rate limits)
            octocrab::Octocrab::default()
        };

        // Get contents of examples directory
        let contents = octocrab
            .repos(owner, repo)
            .get_content()
            .path("examples")
            .send()
            .await;

        match contents {
            Ok(content_items) => {
                let mut examples = Vec::new();
                
                for item in content_items.items {
                    if let octocrab::models::repos::Content::File(file) = item {
                        if file.name.ends_with(".rs") {
                            if let Some(encoded_content) = file.content {
                                // Decode base64 content
                                let decoded_bytes = base64::decode(encoded_content.replace('\n', ""))
                                    .map_err(|e| EgError::GitHubError(format!("Failed to decode file content: {}", e)))?;
                                
                                let content = String::from_utf8(decoded_bytes)
                                    .map_err(|e| EgError::GitHubError(format!("Invalid UTF-8 in file: {}", e)))?;

                                let search_matches = if let Some(regex) = pattern {
                                    self.find_matches(&content, regex)
                                } else {
                                    Vec::new()
                                };

                                examples.push(Example::ExampleInMemory {
                                    filename: file.name,
                                    contents: content,
                                    search_matches,
                                });
                            }
                        }
                    }
                }
                
                Ok(examples)
            }
            Err(_) => {
                // Examples directory not found or other error
                Ok(Vec::new())
            }
        }
    }

    /// Find regex matches in content (same logic as extraction.rs)
    fn find_matches(&self, content: &str, regex: &Regex) -> Vec<SearchRange> {
        let mut matches = Vec::new();
        
        for mat in regex.find_iter(content) {
            let start_pos = mat.start();
            let end_pos = mat.end();
            
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
