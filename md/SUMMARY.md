# Summary

- [Introduction](./introduction.md) <!-- Explain the purpose of the eg library -->

# Design

- [Overview](./design/overview.md) <!-- Explain the big picture API -->
- [Rust details](./design/rust.md) <!-- Explain details of how we do this for Rust -->

# References

<!-- Claude: consult these references for a more detailed look at particular topics. Each reference has a summary of its contents attached. -->

- [Cargo Cache Access and Crate Source Extraction in Rust Libraries](./references/cargo-cache-access.md) <!-- This report provides a comprehensive guide for developing a Rust library focused on advanced dependency analysis and crate source management. The core objective is to enable programmatic interaction with the Rust package ecosystem, encompassing the parsing of project manifests, resolution of semantic versioning constraints, direct access to Cargo's local cache, and the efficient downloading and extraction of crate sources. A forward-looking component addresses the integration with GitHub repositories as a fallback for source discovery. The analysis prioritizes current best practices, robust error handling, and critical performance considerations, including memory management and I/O efficiency. Concrete code examples are provided to illustrate key operations, ensuring practical applicability for developers building sophisticated Rust tooling. -->