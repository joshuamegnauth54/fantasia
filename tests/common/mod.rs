use std::{env, time::Duration};

use reqwest::Client;
use sqlx::PgPool;
use tracing::info;

use fantasia_web::{app::Fantasia, Server};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

// Default user agent
fn user_agent() -> String {
    format!(
        "{}/{} ({}; {})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH
    )
}

#[tracing::instrument(skip(pool))]
pub fn spawn(pool: PgPool) -> Server {
    info!("Spawning server for tests");

    // Bind to any port. This is useful for running multiple apps concurrently for tests
    let listener = "127.0.0.1:0"
        .parse()
        .expect("127.0.0.1:0 is a valid address");

    let fantasia = Fantasia::new(listener, pool);
    fantasia
        .try_into()
        .expect("Building a `Server` from a valid `Fantasia` struct should succeed")
}

#[tracing::instrument]
pub fn test_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(user_agent())
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_TIMEOUT)
        .connection_verbose(true)
        .use_rustls_tls()
        .trust_dns(true)
        .build()
}
