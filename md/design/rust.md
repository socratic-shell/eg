# Rust Implementation Details

## Technical Dependencies

- `cargo_metadata`: Get resolved dependency graph from current project
- `semver`: Parse and match version constraints  
- `home`: Locate cargo cache directory
- `flate2`: Decompress gzipped .crate files
- `tar`: Extract files from tar archives
- `reqwest`: Download .crate files from crates.io
- `crates_io_api`: Query crates.io for available versions and repository metadata
- `octocrab`: GitHub API client for repository fallback

## Version Resolution Implementation

1. **Current project detection**: Use `cargo_metadata` from current working directory to get resolved dependencies
2. **Version constraint resolution**: Query crates.io API for all available versions, then use `semver::VersionReq` to filter for latest matching version
3. **Fallback to latest**: If no current project or dependency not found, use crates.io API to get latest version

## Cache Location Details

- Use `home::cargo_home()` to find `~/.cargo/registry/cache/`
- Construct paths like `cache/github.com-1ecc6299db9ec823/{crate}-{version}.crate`
- Check file existence before attempting download

## Streaming Implementation

Pipeline: HTTP response → `GzDecoder` → `tar::Archive` → filter entries → extract matching files

Download URL format: `https://static.crates.io/crates/{crate_name}/{crate_name}-{version}.crate`

```rust
// Conceptual flow
let download_url = format!("https://static.crates.io/crates/{}/{}-{}.crate", 
                          crate_name, crate_name, version);
let response = reqwest::get(download_url).await?;
let gz_decoder = GzDecoder::new(response);
let mut archive = Archive::new(gz_decoder);

for entry in archive.entries()? {
    let path = entry.path()?;
    if path.starts_with("examples/") {
        // Process this file - it's a gzipped tar archive
    }
}
```

## Source Location Pipeline

1. **Check local cache**: Look in cargo's cache (`~/.cargo/registry/cache/`) first
2. **Download if needed**: Fetch `.crate` file from crates.io
3. **Stream extraction**: Process gzipped tar archive in memory without filesystem extraction
4. **Filter examples**: Extract only files from `examples/` directories
5. **GitHub fallback**: If no examples directory found, extract repository URL from crate metadata and search GitHub

## GitHub Repository Fallback

When no `examples/` directory is found in the crate source:
- Extract repository URL from crate metadata (via `crates_io_api`)
- Parse repository URL to detect if it's GitHub (github.com)
- For GitHub repositories: use GitHub API to search for examples
- Parse GitHub URL to get owner/repo (e.g., `https://github.com/tokio-rs/tokio` → `tokio-rs/tokio`)
- Search `examples/` directory in the GitHub repository

*Future: Support for GitLab, Codeberg, and other Git hosting platforms*

## Search Scope

Initially focused on files in `examples/` directories. Future extensions will include:
- Doc comments throughout the crate
- GitHub repository fallback when crate sources lack examples
