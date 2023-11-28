mod args;
mod config;
mod telemetry;

use std::path::Path;

use anyhow::{Context, Result};
use telemetry::logging;
use tracing::info;
// use tracing_log::LogTracer;

use args::Args;
use config::General;
use fantasia_web::{app::Fantasia, PgPoolOptions};

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    logging().context("Failed to set a global logger")?;

    let args = Args::parse_args().context("Failed to parse arguments")?;
    let conf_path = args
        .conf
        .as_deref()
        .unwrap_or_else(|| Path::new("fantasia.toml"));

    info!("Loading settings");
    let mut config = General::from_path(conf_path).context("Could not load settings")?;
    config::dotenv(config.env_file.as_deref()).context("Invalid .env file")?;
    config.augment(args);
    let db_url = config.postgres.database_url();

    info!("Building Fantasia instance");
    let fantasia = Fantasia::new_from_addr(
        (config.host, config.port),
        PgPoolOptions::default(),
        &db_url,
    )
    .await
    .context("Failed to initialize Fantasia instance")?;

    info!("Starting server");
    fantasia
        .into_server()
        .context("Failed to build Hyper server")?
        .await?;
    Ok(())
}
