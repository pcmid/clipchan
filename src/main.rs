use anyhow::Context;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod api;
mod config;
mod core;
mod server;
mod service;
mod storage;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(name = "clipchan", about = "")]
pub struct CliArgs {
    /// Enable debug mode
    #[arg(short, long, default_value = "info")]
    pub log_level: Level,

    /// Path to configuration file
    #[arg(short, long, default_value = "clipchan.toml")]
    pub config_file: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args = CliArgs::parse();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(args.log_level.into()))
        .init();

    tracing::debug!("Loading configuration from: {}", args.config_file);
    let config = Config::load(args.config_file).context("Failed to load configuration")?;

    tracing::debug!("Configuration loaded successfully: {:?}", config);

    tracing::info!("Server starting...");

    server::run(config)
        .await
        .context("Server failed to start")?;

    Ok(())
}
