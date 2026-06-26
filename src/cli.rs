use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "{{project-name}}",
    version,
    about = "{{project_description}}"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the HTTP server: MCP at /mcp, REST at /api/v1
    Up,
    /// Manage MCP client integrations
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
}

#[derive(Subcommand)]
pub enum McpAction {
    /// Register as an MCP server in a supported client
    Install {
        /// Client to register with: claude, cursor, windsurf
        client: String,
    },
}

pub fn cmd_mcp_install(client: &str) -> anyhow::Result<()> {
    match client {
        "claude" => {
            println!("Add to ~/.claude/claude_desktop_config.json:");
            println!(
                r#"{{{{
  "mcpServers": {{{{
    "{{project-name}}": {{{{
      "command": "{{project-name}}",
      "args": []
    }}}}
  }}}}
}}}}"#
            );
        }
        other => {
            anyhow::bail!("unsupported client: {other}. Supported: claude, cursor, windsurf");
        }
    }
    Ok(())
}
