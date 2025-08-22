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
- `grep` or `ripgrep`: Fast text searching through extracted files

## Version Resolution Implementation

1. **Current project detection**: Use `cargo_metadata` from current working directory to get resolved dependencies
2. **Version constraint resolution**: Query crates.io API for all available versions, then use `semver::VersionReq` to filter for latest matching version
3. **Fallback to latest**: If no current project or dependency not found, use crates.io API to get latest version

## Cache Location Details

- Use `home::cargo_home()` to find `~/.cargo/registry/cache/`
- Construct paths like `cache/github.com-1ecc6299db9ec823/{crate}-{version}.crate`
- Check file existence before attempting download

## Extraction and Search Pipeline

**New simplified approach**: Extract full crate to local cache, then search everything

```rust
// Conceptual flow
1. Check if crate is already extracted in our cache
2. If not, download .crate file and extract to cache directory
3. Use grep/ripgrep to search all files for pattern
4. Categorize results: examples/ vs other files
5. Return paths and context, not file contents
```

## Local Cache Structure

```
~/.cache/eg/  (or platform equivalent)
├── extractions/
│   ├── serde-1.0.197/     # Full crate extraction
│   │   ├── src/
│   │   ├── examples/
│   │   └── ...
│   └── tokio-1.35.0/
└── metadata/
    └── extraction_info.json  # Track what's cached
```

## Source Location Pipeline

1. **Check local extraction cache**: Look for already-extracted crate
2. **Check cargo cache**: Look in cargo's cache (`~/.cargo/registry/cache/`) for .crate file
3. **Download if needed**: Fetch `.crate` file from crates.io
4. **Extract to cache**: Decompress and extract full crate to our cache directory
5. **Search with grep**: Use fast text search across all files
6. **GitHub fallback**: If no examples found, search GitHub repository

## GitHub Repository Fallback

When no examples are found in the extracted crate:
- Extract repository URL from crate metadata (via `crates_io_api`)
- Parse repository URL to detect if it's GitHub (github.com)
- For GitHub repositories: use GitHub API to search for examples
- Parse GitHub URL to get owner/repo (e.g., `https://github.com/tokio-rs/tokio` → `tokio-rs/tokio`)
- Search `examples/` directory in the GitHub repository

*Future: Support for GitLab, Codeberg, and other Git hosting platforms*

## Search Implementation

- Use `ripgrep` or similar for fast text search
- Search all `.rs` files in the extraction
- Categorize results by directory (examples/ vs src/ vs tests/ etc.)
- Include configurable context lines around matches
- Return file paths relative to extraction root
