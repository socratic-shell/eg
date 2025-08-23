# eg-mcp: MCP Server for the eg Library

This directory contains an MCP (Model Context Protocol) server that exposes the eg library functionality to LLMs and AI assistants.

## Features

The `eg-mcp` server provides the following MCP tools:

### `search_crate_examples`
Search for patterns in Rust crate examples and source code.

**Parameters:**
- `crate_name` (string): Name of the crate to search
- `pattern` (string, optional): Regex pattern to search for

**Example:**
```json
{
  "name": "search_crate_examples",
  "arguments": {
    "crate_name": "tokio",
    "pattern": "spawn"
  }
}
```

### `get_crate_source`
Get the full path to an extracted crate for detailed exploration.

**Parameters:**
- `crate_name` (string): Name of the crate

**Example:**
```json
{
  "name": "get_crate_source",
  "arguments": {
    "crate_name": "serde"
  }
}
```

## Building and Running

### Prerequisites
- Rust toolchain
- The eg library dependencies

### Build
```bash
cargo build --bin eg-mcp --features mcp
```

### Run
```bash
cargo run --bin eg-mcp --features mcp
```

The server communicates via stdio using the MCP protocol.

### Testing with MCP Inspector
```bash
npx @modelcontextprotocol/inspector cargo run --bin eg-mcp --features mcp
```

## Integration

To use this server with an MCP client, configure it to run:
```bash
cargo run --bin eg-mcp --features mcp
```

The server will:
1. Initialize with MCP protocol version 2024-11-05
2. Expose the two tools described above
3. Handle tool calls by delegating to the eg library
4. Return formatted results with code examples and file paths

## Architecture

The MCP server is implemented as a binary target within the eg crate, using:
- **rmcp**: Official Rust MCP SDK
- **eg library**: Core functionality for crate searching and extraction
- **serde**: JSON serialization for MCP protocol
- **tokio**: Async runtime

The server follows the standard MCP server pattern:
1. Tool definitions with JSON schema validation
2. Async tool handlers that delegate to eg library
3. Formatted responses optimized for LLM consumption
