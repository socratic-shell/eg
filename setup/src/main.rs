#!/usr/bin/env cargo
//! eg Library MCP Server Setup Tool
//!
//! Builds and configures the eg-mcp server for use with AI assistants.

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, ValueEnum)]
enum CLITool {
    #[value(name = "q")]
    QCli,
    #[value(name = "claude")]
    ClaudeCode,
    #[value(name = "both")]
    Both,
    #[value(name = "auto")]
    Auto,
}

#[derive(Debug, Clone, ValueEnum)]
enum ClaudeScope {
    #[value(name = "user")]
    User,
    #[value(name = "local")]
    Local,
    #[value(name = "project")]
    Project,
}

#[derive(Parser)]
#[command(
    name = "setup",
    about = "Build and configure eg-mcp server for AI assistants",
    long_about = r#"
Build and configure eg-mcp server for AI assistants

This tool builds the eg-mcp server and configures it for use with Q CLI or Claude Code.
The server provides access to Rust crate examples and source code through the Model Context Protocol.

Examples:
  cargo setup                           # Install to PATH and setup for production use
  cargo setup --dev                     # Build in target/ for development
  cargo setup --tool q                  # Setup for Q CLI only
  cargo setup --tool claude             # Setup for Claude Code only
  cargo setup --tool both               # Setup for both tools

Prerequisites:
  - Rust and Cargo (https://rustup.rs/)
  - Q CLI or Claude Code
"#
)]
struct Args {
    /// Which CLI tool to configure
    #[arg(long, default_value = "auto")]
    tool: CLITool,

    /// Scope for Claude Code MCP configuration
    #[arg(long, default_value = "user")]
    claude_scope: ClaudeScope,

    /// Use development mode (build in target/ directory instead of installing to PATH)
    #[arg(long)]
    dev: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("📚 eg Library MCP Server Setup");
    println!("{}", "=".repeat(32));

    // Determine which tool to use
    let tool = match args.tool {
        CLITool::Auto => detect_available_tools()?,
        other => other,
    };

    // Check prerequisites
    check_rust()?;

    match tool {
        CLITool::QCli => check_q_cli()?,
        CLITool::ClaudeCode => check_claude_code()?,
        CLITool::Both => {
            check_q_cli()?;
            check_claude_code()?;
        }
        CLITool::Auto => unreachable!("Auto should have been resolved earlier"),
    }

    // Build the MCP server
    let binary_path = if args.dev {
        build_mcp_server()?
    } else {
        install_mcp_server()?
    };

    // Setup MCP server(s)
    let success = match tool {
        CLITool::QCli => {
            setup_q_cli_mcp(&binary_path)?
        }
        CLITool::ClaudeCode => {
            setup_claude_code_mcp(&binary_path, &args.claude_scope)?
        }
        CLITool::Both => {
            setup_q_cli_mcp(&binary_path)?
                && setup_claude_code_mcp(&binary_path, &args.claude_scope)?
        }
        CLITool::Auto => unreachable!("Auto should have been resolved earlier"),
    };

    if success {
        print_next_steps(&tool, args.dev)?;
    } else {
        println!("\n❌ Setup incomplete. Please fix the errors above and try again.");
        std::process::exit(1);
    }

    Ok(())
}

fn check_rust() -> Result<()> {
    if which::which("cargo").is_err() {
        return Err(anyhow!(
            "❌ Error: Cargo not found. Please install Rust first.\n   Visit: https://rustup.rs/"
        ));
    }
    Ok(())
}

fn check_q_cli() -> Result<()> {
    if which::which("q").is_err() {
        return Err(anyhow!(
            "❌ Error: Q CLI not found. Please install Q CLI first.\n   Visit: https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/q-cli.html"
        ));
    }
    Ok(())
}

fn check_claude_code() -> Result<()> {
    if !is_claude_available() {
        return Err(anyhow!(
            "❌ Error: Claude Code not found. Please install Claude Code first.\n   Visit: https://claude.ai/code"
        ));
    }
    Ok(())
}

fn is_claude_available() -> bool {
    which::which("claude").is_ok()
        || home::home_dir().map_or(false, |home| home.join(".claude").exists())
}

fn detect_available_tools() -> Result<CLITool> {
    let has_q = which::which("q").is_ok();
    let has_claude = is_claude_available();

    match (has_q, has_claude) {
        (true, true) => Ok(CLITool::Both),
        (true, false) => Ok(CLITool::QCli),
        (false, true) => Ok(CLITool::ClaudeCode),
        (false, false) => Err(anyhow!(
            "❌ No supported CLI tools found. Please install Q CLI or Claude Code.\n   Q CLI: https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/q-cli.html\n   Claude Code: https://claude.ai/code"
        )),
    }
}

fn get_repo_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").context(
        "❌ Setup tool must be run via cargo (e.g., 'cargo setup'). CARGO_MANIFEST_DIR not found.",
    )?;

    let manifest_path = PathBuf::from(manifest_dir);
    // If we're in setup/, go up to repo root
    if manifest_path.file_name() == Some(std::ffi::OsStr::new("setup")) {
        if let Some(parent) = manifest_path.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(manifest_path)
}

fn install_mcp_server() -> Result<PathBuf> {
    let repo_root = get_repo_root()?;

    println!("📦 Installing eg-mcp server to PATH...");
    println!("   Installing from: {}", repo_root.display());

    let output = Command::new("cargo")
        .args(["install", "--path", ".", "--bin", "eg-mcp", "--force"])
        .current_dir(&repo_root)
        .output()
        .context("Failed to execute cargo install")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "❌ Failed to install eg-mcp server:\n   Error: {}",
            stderr.trim()
        ));
    }

    // Verify the binary is accessible
    if which::which("eg-mcp").is_err() {
        println!("⚠️  Warning: eg-mcp not found in PATH after installation");
        if let Some(home) = home::home_dir() {
            let cargo_bin = home.join(".cargo").join("bin");
            println!(
                "   Make sure {} is in your PATH environment variable",
                cargo_bin.display()
            );
        }
    }

    println!("✅ eg-mcp server installed successfully!");
    Ok(PathBuf::from("eg-mcp"))
}

fn build_mcp_server() -> Result<PathBuf> {
    let repo_root = get_repo_root()?;

    println!("🔨 Building eg-mcp server for development...");
    println!("   Building in: {}", repo_root.display());

    let output = Command::new("cargo")
        .args(["build", "--release", "--bin", "eg-mcp"])
        .current_dir(&repo_root)
        .output()
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "❌ Failed to build eg-mcp server:\n   Error: {}",
            stderr.trim()
        ));
    }

    let binary_path = repo_root.join("target").join("release").join("eg-mcp");
    if !binary_path.exists() {
        return Err(anyhow!(
            "❌ Build verification failed: Built binary not found at {}",
            binary_path.display()
        ));
    }

    println!("✅ eg-mcp server built successfully!");
    Ok(binary_path)
}

fn setup_q_cli_mcp(binary_path: &Path) -> Result<bool> {
    println!("🔧 Registering eg-mcp server with Q CLI...");
    println!("   Binary path: {}", binary_path.display());

    let output = Command::new("q")
        .args([
            "mcp",
            "add",
            "--name",
            "eg",
            "--command",
            &binary_path.to_string_lossy(),
            "--force",
        ])
        .output()
        .context("Failed to execute q mcp add")?;

    if output.status.success() {
        println!("✅ MCP server 'eg' registered successfully with Q CLI!");
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Failed to register MCP server with Q CLI:");
        println!("   Error: {}", stderr.trim());
        Ok(false)
    }
}

fn setup_claude_code_mcp(binary_path: &Path, scope: &ClaudeScope) -> Result<bool> {
    let scope_str = match scope {
        ClaudeScope::User => "user",
        ClaudeScope::Local => "local",
        ClaudeScope::Project => "project",
    };

    println!("🔧 Registering eg-mcp server with Claude Code...");
    println!("   Binary path: {}", binary_path.display());
    println!("   Scope: {}", scope_str);

    let output = Command::new("claude")
        .args([
            "mcp",
            "add",
            "--scope",
            scope_str,
            "eg",
            &binary_path.to_string_lossy(),
        ])
        .output()
        .context("Failed to execute claude mcp add")?;

    if output.status.success() {
        println!("✅ MCP server 'eg' registered successfully with Claude Code!");
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Failed to register MCP server with Claude Code:");
        println!("   Error: {}", stderr.trim());

        if stderr.contains("already exists") {
            println!("\n💡 Tip: Remove existing server with: claude mcp remove eg");
        }

        Ok(false)
    }
}

fn print_next_steps(tool: &CLITool, dev_mode: bool) -> Result<()> {
    if dev_mode {
        println!("\n🎉 Development setup complete! eg-mcp server is ready for development.");
        println!("🔧 Running in development mode - server will use target/release/eg-mcp");
    } else {
        println!("\n🎉 Production setup complete! eg-mcp server is installed and ready.");
        println!("📦 Server installed to PATH as 'eg-mcp'");
    }

    match tool {
        CLITool::QCli | CLITool::Both => {
            println!("\n🧪 Test with Q CLI:");
            println!("   q chat \"Search for examples of tokio::spawn in the tokio crate\"");
            println!("   q chat \"Get the source path for the serde crate\"");
        }
        _ => {}
    }

    match tool {
        CLITool::ClaudeCode | CLITool::Both => {
            println!("\n🧪 Test with Claude Code:");
            println!("   claude chat \"Search for examples of tokio::spawn in the tokio crate\"");
            println!("   claude chat \"Get the source path for the serde crate\"");
        }
        _ => {}
    }

    println!("\n📝 Available MCP tools:");
    println!("- search_crate_examples: Search for patterns in Rust crate examples and source");
    println!("- get_crate_source: Get the full path to an extracted crate");

    if dev_mode {
        println!("\n🔧 Development workflow:");
        println!("- For server changes: cargo build --release --bin eg-mcp");
        println!("- Test changes: cargo test --test mcp_integration");
    }

    Ok(())
}
