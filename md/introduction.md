# The `eg` library

*Part of the [Socratic Shell](https://socratic-shell.github.io/socratic-shell/) project.*

The `eg` library provides programmatic access to libraries examples and documentation. It downloads and searches through library sources to find usage examples, with a focus on serving both human developers and LLMs that need concrete examples when generating code.

Our initial focus is Rust code, but we expand to expand to other languages.

Key capabilities:
- Download and cache library sources from crates.io
- Search through examples directories and doc comments
- Handle version resolution and dependency analysis
- Provide streaming extraction for memory-efficient processing of large crates
- Future: fallback to GitHub repository searching when crate sources lack examples
