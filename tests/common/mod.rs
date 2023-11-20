use sqlx::PgPool;
use tracing::info;

use fantasia_web::{app::Fantasia, Server};

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
