mod args;
mod config;
mod telemetry;

use anyhow::{Context, Result};
use config::General;
use fantasia_web::{app::Fantasia, PgPoolOptions};
// use tracing_log::LogTracer;

use secrecy::ExposeSecret;
use telemetry::logging;
use tracing::info;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    logging().context("Failed to set a global logger")?;

    info!("Loading settings");
    let config = General::from_path("fantasia.toml").context("Could not load settings")?;
    let db_url = config.postgres.database_url();

    info!("Building Fantasia instance");
    let fantasia = Fantasia::new_from_addr(
        (config.host, config.port),
        PgPoolOptions::default(),
        db_url.expose_secret(),
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
