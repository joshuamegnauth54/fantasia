use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use tracing::trace;

/// Health and sanity check endpoint.
#[tracing::instrument(level = "debug")]
pub async fn health_check(ConnectInfo(addr): ConnectInfo<SocketAddr>) {
    trace!("Connected: {addr}")
}
