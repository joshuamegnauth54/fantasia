mod args;
mod config;
mod pool_options;
mod telemetry;

use std::path::Path;

use anyhow::{Context, Result};
use futures::future::join_all;
use telemetry::logging;
use tracing::info;
// use tracing_log::LogTracer;

use args::Args;
use config::Config;
use fantasia_web::{app::FantasiaBuilder, PgPoolOptions};

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
    let mut config = Config::from_path(conf_path).context("Could not load settings")?;
    config::dotenv(config.fantasia.env_file.as_deref()).context("Invalid .env file")?;
    config.augment(args);
    let db_url = config.postgres.database_url_view();

    info!("Building Fantasia instance");
    let fantasia = FantasiaBuilder::new_from_addr(
        (config.fantasia.host, config.fantasia.port),
        PgPoolOptions::default(),
        &db_url,
    )
    .await
    .context("Failed to initialize Fantasia instance")?;

    info!("Starting server");
    let results = join_all(
        fantasia
            .into_server()
            // .context("Failed to build Hyper server")?
            .await
            .into_iter()
            .map(|sock_res| async {
                sock_res
                    .expect("Binding to a local socket should succeed")
                    .server
                    .await
            }),
    )
    .await;

    for result in results {
        result.context("Spawned Fantasia instance crashed")?;
    }

    Ok(())
}
