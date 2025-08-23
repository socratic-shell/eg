use anyhow::Result;
use eg_mcp::EgMcpServer;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

/// MCP server for the eg library
/// Usage: eg-mcp
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting eg MCP server");

    // Create and serve the MCP server
    let service = EgMcpServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}

mod eg_mcp {
    use eg::Eg;
    use rmcp::{
        ErrorData as McpError, RoleServer, ServerHandler,
        handler::server::{router::tool::ToolRouter, tool::Parameters},
        model::*,
        schemars,
        service::RequestContext,
        tool, tool_handler, tool_router,
    };
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct SearchCrateExamplesRequest {
        /// Name of the crate to search
        pub crate_name: String,
        /// Optional search pattern (regex)
        pub pattern: Option<String>,
    }

    #[derive(Debug, Deserialize, schemars::JsonSchema)]
    pub struct GetCrateSourceRequest {
        /// Name of the crate
        pub crate_name: String,
    }

    #[derive(Clone)]
    pub struct EgMcpServer {
        tool_router: ToolRouter<EgMcpServer>,
    }

    #[tool_router]
    impl EgMcpServer {
        pub fn new() -> Self {
            Self {
                tool_router: Self::tool_router(),
            }
        }

        #[tool(description = "Search for patterns in Rust crate examples and source code")]
        async fn search_crate_examples(
            &self,
            Parameters(SearchCrateExamplesRequest { crate_name, pattern }): Parameters<SearchCrateExamplesRequest>,
        ) -> Result<CallToolResult, McpError> {
            let mut search = Eg::rust_crate(&crate_name);
            
            if let Some(pattern) = pattern {
                search = search.pattern(&pattern).map_err(|e| {
                    let error_msg = format!("Invalid regex pattern: {}", e);
                    McpError::invalid_params(error_msg, None)
                })?;
            }

            match search.search().await {
                Ok(result) => {
                    let response = serde_json::to_string_pretty(&result).unwrap();
                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => {
                    let error_msg = format!("Search failed: {}", e);
                    Err(McpError::internal_error(
                        error_msg,
                        Some(json!({
                            "crate_name": crate_name,
                            "error": e.to_string()
                        })),
                    ))
                }
            }
        }

        #[tool(description = "Get the full path to an extracted crate for detailed exploration")]
        async fn get_crate_source(
            &self,
            Parameters(GetCrateSourceRequest { crate_name }): Parameters<GetCrateSourceRequest>,
        ) -> Result<CallToolResult, McpError> {
            match Eg::rust_crate(&crate_name).search().await {
                Ok(result) => {
                    let response = json!({
                        "crate_name": crate_name,
                        "version": result.version,
                        "checkout_path": result.checkout_path.to_string_lossy(),
                        "message": format!("Crate {} v{} extracted to {}", 
                                         crate_name, result.version, result.checkout_path.display())
                    });
                    Ok(CallToolResult::success(vec![Content::text(response.to_string())]))
                }
                Err(e) => {
                    let error_msg = format!("Failed to extract crate: {}", e);
                    Err(McpError::internal_error(
                        error_msg,
                        Some(json!({
                            "crate_name": crate_name,
                            "error": e.to_string()
                        })),
                    ))
                }
            }
        }
    }

    #[tool_handler]
    impl ServerHandler for EgMcpServer {
        fn get_info(&self) -> ServerInfo {
            ServerInfo {
                protocol_version: ProtocolVersion::V_2024_11_05,
                capabilities: ServerCapabilities::builder()
                    .enable_tools()
                    .build(),
                server_info: Implementation::from_build_env(),
                instructions: Some(
                    "This server provides access to the eg library for searching Rust crate examples and source code. \
                     Use 'search_crate_examples' to find patterns in crate code, and 'get_crate_source' to get the path \
                     to extracted crate source for detailed exploration.".to_string()
                ),
            }
        }

        async fn initialize(
            &self,
            _request: InitializeRequestParam,
            _context: RequestContext<RoleServer>,
        ) -> Result<InitializeResult, McpError> {
            Ok(self.get_info())
        }
    }
}
