mod common;
use common::{spawn, test_client};

use sqlx::PgPool;
use test_log::test;
use tracing::info;

#[tracing::instrument(skip(pool))]
#[test(sqlx::test)]
async fn health_check_works(pool: PgPool) {
    let app = spawn(pool);

    let endpoint = format!("http://{}/health_check", app.local_addr());
    info!("Sending a GET request to {endpoint}");
    let _handle = tokio::spawn(app);

    let response = test_client()
        .expect("Should be able to build an HTTP client")
        .get(&*endpoint)
        .send()
        .await
        .unwrap_or_else(|e| panic!("Should be able to send a GET request ({endpoint})\n\r{e}"));
    assert_eq!(200, response.status());
}
