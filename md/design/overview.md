# Design Overview

## Purpose

The `eg` library helps developers and LLMs find concrete usage examples for library APIs. When generating code, it's often unclear how to properly use a library's functions - `eg` solves this by extracting and searching real examples from library sources.

## API Design

The library uses a builder pattern for flexible queries:

```rust
// Basic usage - find examples in current project's tokio dependency
let result: SearchResult = Eg::rust_crate("tokio")
    .search().await?;

// With version constraint
let result: SearchResult = Eg::rust_crate("serde")
    .version("^1.0")
    .search().await?;

// With pattern matching
let result: SearchResult = Eg::rust_crate("tokio")
    .pattern(r"spawn")
    .search().await?;
```

## Result Types

Search results include metadata about the search and distinguish between locally available and downloaded examples:

```rust
struct SearchResult {
    /// The exact version that was searched
    version: String,
    /// Total number of example files found
    total_examples: usize,
    /// Number of examples that matched the search pattern
    matched_examples: usize,
    /// The actual example files and their matches
    examples: Vec<Example>,
}

enum Example {
    ExampleOnDisk {
        path: PathBuf,    // Local file path
        search_matches: Vec<SearchRange>,
    },
    ExampleInMemory {
        filename: String, // Original path in examples/
        contents: String, // File contents
        search_matches: Vec<SearchRange>,
    }
}

struct SearchRange {
    /// 0-based byte that is the start of the match
    byte_start: u32,

    /// 1-based line where the search started
    line_start: u32,

    /// 1-based column marking the end of the match (exclusive)
    column_start: u32,

    /// 0-based byte that is the end of the match
    byte_end: u32,

    /// 1-based line where the search ended
    line_end: u32,

    /// 1-based line marking the end of the match (exclusive)
    column_end: u32,
}
```

## Version Resolution Strategy

1. **Explicit version**: If `.version()` is specified, find the latest version matching that constraint
2. **Current project**: If no version specified, look for the library in the current project's dependencies
3. **Latest fallback**: If no current project, use the latest version available from the package registry


