# Design Overview

## Purpose

The `eg` library helps developers and LLMs find concrete usage examples for library APIs. When generating code, it's often unclear how to properly use a library's functions - `eg` solves this by extracting complete crate sources and providing comprehensive search results with context.

**Key insight**: Instead of trying to be selective about what to extract, we extract the full crate and let users explore comprehensively. This gives LLMs access to examples, tests, documentation, and internal usage patterns - providing richer context for code generation.

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

// With pattern matching and context
let result: SearchResult = Eg::rust_crate("tokio")
    .pattern(r"spawn")?
    .context_lines(3)  // 3 lines before/after each match
    .search().await?;

// Access results
println!("Crate extracted to: {}", result.checkout_path.display());
println!("Found {} example matches, {} other matches", 
         result.example_matches.len(), result.other_matches.len());

// LLM can request more context from specific files as needed
let file_content = std::fs::read_to_string(
    result.checkout_path.join(&result.example_matches[0].file_path)
)?;
```

## Result Types

Search results provide access to the full crate extraction and categorized matches:

```rust
struct SearchResult {
    /// The exact version that was searched
    version: String,
    /// Path to the full crate extraction on disk
    checkout_path: PathBuf,
    /// Matches found in examples/ directory
    example_matches: Vec<Match>,
    /// Matches found elsewhere in the crate
    other_matches: Vec<Match>,
}

struct Match {
    /// Relative path within the crate
    file_path: PathBuf,
    /// 1-based line number where match was found
    line_number: u32,
    /// The line containing the match
    line_content: String,
    /// Lines before the match for context
    context_before: Vec<String>,
    /// Lines after the match for context
    context_after: Vec<String>,
}
```
```

## Version Resolution Strategy

1. **Explicit version**: If `.version()` is specified, find the latest version matching that constraint
2. **Current project**: If no version specified, look for the library in the current project's dependencies
3. **Latest fallback**: If no current project, use the latest version available from the package registry


