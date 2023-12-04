mod common;

use std::future::IntoFuture;

use futures::future::join_all;
use reqwest::StatusCode;
use sqlx::PgPool;
use test_log::test;
use tracing::info;

use common::{spawn, test_client};
use fantasia_web::app::Fantasia;

#[tracing::instrument(skip(pool))]
#[test(sqlx::test)]
async fn health_check_works(pool: PgPool) {
    let (endpoints, servers): (Vec<_>, Vec<_>) = spawn(pool)
        .await
        .into_iter()
        .map(|sock_res| {
            let Fantasia { sock_addr, server } =
                sock_res.expect("Binding to a local socket for tests should succeed.");
            let endpoint = format!("http://{}/health_check", sock_addr);
            (endpoint, server)
        })
        .unzip();

    // Spawn servers
    if servers.is_empty() {
        panic!("Expected at least one spawned Fantasia instance");
    }
    let servers = join_all(servers.into_iter().map(IntoFuture::into_future));
    let _handle = tokio::spawn(servers);

    let client = test_client().expect("Should be able to build an HTTP client");

    for endpoint in endpoints {
        info!("Sending a GET request to {endpoint}");
        let response =
            client.get(&*endpoint).send().await.unwrap_or_else(|e| {
                panic!("Should be able to send a GET request ({endpoint})\n\r{e}")
            });
        assert_eq!(StatusCode::OK, response.status());
    }
}
