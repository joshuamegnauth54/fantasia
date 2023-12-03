mod common;
use common::{spawn, test_client, TestServer};

use std::future::IntoFuture;

use reqwest::StatusCode;
use sqlx::PgPool;
use test_log::test;
use tracing::info;

#[tracing::instrument(skip(pool))]
#[test(sqlx::test)]
async fn health_check_works(pool: PgPool) {
    let TestServer { sock_addr, server } = spawn(pool)
        .await
        .into_iter()
        .map(|sock_res| sock_res.expect("Binding to a local socket for tests should succeed."))
        .next()
        .expect("Should have at least one server instance");

    let endpoint = format!("http://{}/health_check", sock_addr);
    info!("Sending a GET request to {endpoint}");
    let _handle = tokio::spawn(server.into_future());

    let response = test_client()
        .expect("Should be able to build an HTTP client")
        .get(&*endpoint)
        .send()
        .await
        .unwrap_or_else(|e| panic!("Should be able to send a GET request ({endpoint})\n\r{e}"));
    assert_eq!(StatusCode::OK, response.status());
}
