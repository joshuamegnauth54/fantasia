use std::{env, io, net::SocketAddr, time::Duration};

use reqwest::Client;
use sqlx::PgPool;
use tracing::info;

use fantasia_web::{app::Fantasia, Serve};

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

#[derive(Debug)]
pub struct TestServer {
    pub sock_addr: SocketAddr,
    pub server: Serve,
}

#[tracing::instrument(skip(pool))]
pub async fn spawn(pool: PgPool) -> Vec<io::Result<TestServer>> {
    info!("Spawning server for tests");

    // Bind to any port. This is useful for running multiple apps concurrently for tests
    let sockets = ["127.0.0.1:0"
        .parse()
        .expect("`127.0.0.1:0` is a valid address")];

    let fantasia = Fantasia::new(&sockets, pool);
    fantasia
        .into_server()
        .await
        .into_iter()
        .zip(sockets)
        .map(|(res, addr)| {
            res.map(|server| TestServer {
                sock_addr: addr,
                server,
            })
        })
        .collect()
    // .expect("Building a `Server` from a valid `Fantasia` struct should succeed")
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
