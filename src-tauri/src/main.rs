#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "sgame-studio", about = "SGameStudio - All-in-one AI-Native Game Asset Creation Studio", version)]
pub struct Cli {
    /// Run in server mode
    #[arg(short = 's', long)]
    pub server: bool,

    /// Server port
    #[arg(short = 'p', long, default_value = "8080")]
    pub port: u16,

    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Run MCP stdio protocol
    #[arg(short = 'm', long)]
    pub mcp: bool,

    /// Enable MCP SSE events (implies --server)
    #[arg(long)]
    pub mcp_sse: bool,

    /// Disable GPU acceleration
    #[arg(long)]
    pub no_gpu: bool,

    /// Theme name
    #[arg(short = 't', long, value_parser = ["obsidian", "cyberpunk", "paper", "high-contrast"])]
    pub theme: Option<String>,

    /// Config file path
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run a script
    RunScript {
        script: String,
        #[arg(long)]
        export_to: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Export project
    Export { project: PathBuf },
    /// Run self diagnostics
    Test,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Some(Command::Test) = cli.command {
        return sgame_studio::run_self_diagnostics();
    }

    if cli.mcp {
        return sgame_studio::mcp::run_stdio();
    }

    if cli.server || cli.mcp_sse {
        let runtime = tokio::runtime::Runtime::new()?;
        let host: std::net::IpAddr = cli.host.parse().map_err(|e| anyhow::anyhow!("invalid host: {e}"))?;
        let bus = std::sync::Arc::new(sgame_studio::core::CommandBus::new(sgame_studio::core::SceneState::new()).0);
        runtime.block_on(sgame_studio::server::run_server(bus, host, cli.port))?;
    } else {
        #[cfg(feature = "gui")]
        {
            // In the desktop app we must also expose the backend REST/WS/MCP-SSE
            // server on the loopback port so the UI's health check, WebSocket,
            // settings API and AI routing all work (previously only --server spun
            // it up, which is why the UI showed "disconnected" and had nowhere to
            // save AI provider keys). The server runs on its own Tokio runtime in
            // a background thread so it never blocks the WebView event loop.
            let port = sgame_studio::server::default_port();
            std::thread::spawn(move || {
                let runtime = match tokio::runtime::Runtime::new() {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("failed to create tokio runtime: {e}");
                        return;
                    }
                };
                let bus = std::sync::Arc::new(sgame_studio::core::CommandBus::new(sgame_studio::core::SceneState::new()).0);
                let host: std::net::IpAddr = cli.host.clone().parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                tracing::info!("GUI mode: starting embedded backend on {host}:{port}");
                if let Err(e) = runtime.block_on(sgame_studio::server::run_server(bus, host, port)) {
                    tracing::error!("embedded backend error: {e}");
                }
            });

            tauri::Builder::default()
                .run(tauri::generate_context!())
                .map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;
        }
        #[cfg(not(feature = "gui"))]
        {
            println!("GUI disabled in this build; use --server or --mcp");
        }
    }

    Ok(())
}