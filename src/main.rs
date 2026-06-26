use anyhow::Result;
use clap::Parser;
use {{crate_name}}::{cli::{Cli, Command, McpAction}, config, http, server::{{project-name | pascal_case}}};
use rmcp::ServiceExt;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None | Some(Command::Up) => run_up(),
        Some(Command::Mcp { action }) => match action {
            McpAction::Install { client } => {{crate_name}}::cli::cmd_mcp_install(&client),
        },
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "{{project-name}}=info".into()),
        )
        .init();
}

#[tokio::main]
async fn run_up() -> Result<()> {
    init_tracing();
    let settings = config::load_server_settings(&config::global_config_path())?;
    http::run_up(&settings).await
}
