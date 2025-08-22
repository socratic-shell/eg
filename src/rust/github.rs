//! GitHub repository fallback for finding examples

use crate::{Result, EgError, Match};
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use std::env;
use std::path::PathBuf;

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
        _version: &str,
        pattern: Option<&Regex>,
    ) -> Result<Vec<Match>> {
        // Only search if we have a pattern
        let pattern = match pattern {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

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
        let client = crates_io_api::AsyncClient::new(
            "eg-library (https://github.com/socratic-shell/eg)",
            std::time::Duration::from_millis(1000),
        ).map_err(|e| EgError::Other(e.to_string()))?;

        let crate_info = client.get_crate(crate_name).await
            .map_err(|_| EgError::CrateNotFound(crate_name.to_string()))?;

        crate_info.crate_data.repository
            .ok_or_else(|| EgError::NoRepositoryUrl(crate_name.to_string()))
    }

    /// Check if URL is a GitHub repository
    fn is_github_url(&self, url: &str) -> bool {
        url.contains("github.com")
    }

    /// Parse GitHub URL to extract owner and repository name
    fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches(".git");
        let parts: Vec<&str> = url.split('/').collect();
        
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2].to_string();
            let repo = parts[parts.len() - 1].to_string();
            Ok((owner, repo))
        } else {
            Err(EgError::InvalidGitHubUrl(url.to_string()))
        }
    }

    /// Search examples directory in GitHub repository
    async fn search_github_examples(
        &self,
        owner: &str,
        repo: &str,
        pattern: &Regex,
    ) -> Result<Vec<Match>> {
        // Try to get GitHub token from environment
        let token = env::var("GITHUB_TOKEN").ok();
        
        let octocrab = if let Some(token) = token {
            octocrab::Octocrab::builder()
                .personal_token(token)
                .build()?
        } else {
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
                let mut matches = Vec::new();
                
                for item in content_items.items {
                    // Check if this is a file (not a directory or symlink)
                    if item.r#type == "file" && item.name.ends_with(".rs") {
                        if let Some(encoded_content) = &item.content {
                            // Decode base64 content
                            let decoded_bytes = general_purpose::STANDARD
                                .decode(encoded_content.replace('\n', ""))?;
                            
                            let content = String::from_utf8(decoded_bytes)?;

                            // Search for matches in this file
                            let file_matches = self.find_matches_in_content(
                                &PathBuf::from(format!("examples/{}", item.name)),
                                &content,
                                pattern,
                                2, // Default context lines
                            );
                            
                            matches.extend(file_matches);
                        }
                    }
                }
                
                Ok(matches)
            }
            Err(_) => {
                // Examples directory not found or other error
                Ok(Vec::new())
            }
        }
    }

    /// Find regex matches in content and return Match objects
    fn find_matches_in_content(
        &self,
        file_path: &PathBuf,
        content: &str,
        pattern: &Regex,
        context_lines: usize,
    ) -> Vec<Match> {
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            if pattern.is_match(line) {
                let line_number = (line_idx + 1) as u32; // 1-based line numbers
                
                // Get context lines
                let context_start = line_idx.saturating_sub(context_lines);
                let context_end = std::cmp::min(line_idx + context_lines + 1, lines.len());
                
                let context_before = lines[context_start..line_idx]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                
                let context_after = lines[line_idx + 1..context_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                matches.push(Match {
                    file_path: file_path.clone(),
                    line_number,
                    line_content: line.to_string(),
                    context_before,
                    context_after,
                });
            }
        }

        matches
    }
}
