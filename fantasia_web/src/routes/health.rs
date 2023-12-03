use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use tracing::trace;
use uuid::Uuid;

/// Health and sanity check endpoint.
#[tracing::instrument(level = "debug", fields(
    request_id = %Uuid::new_v4()
))]
pub async fn health_check(ConnectInfo(addr): ConnectInfo<SocketAddr>) {
    trace!("Connected: {addr}")
}
