mod config;

use anyhow::{Context, Result};
use fantasia_web::{app::Fantasia, PgPoolOptions};
use config::{General, Postgres};
// use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> Result<()> {
    // LogTracer::init().context("Failed to initialize LogTracer")?;
    tracing_subscriber::fmt::init();
    dotenvy::from_path("dev.env").unwrap();

    let maybe_url = std::env::var("DATABASE_URL").ok();
    let db_url = maybe_url.as_deref().unwrap();

    let fantasia =
        Fantasia::new_from_addr("127.0.0.1:8000", PgPoolOptions::default(), db_url)
            .await
            .context("Failed to initialize Fantasia instance")?;

    fantasia
        .into_server()
        .context("Failed to build Hyper server")?
        .await?;
    Ok(())
}
