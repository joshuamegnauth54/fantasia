pub mod app;
pub mod routes;
pub mod state;

// Reexports
pub use axum::http::StatusCode;
pub use sqlx::{postgres::PgPoolOptions, PgPool};

use std::net::SocketAddr;

use axum::{
    extract::{connect_info::IntoMakeServiceWithConnectInfo, ConnectInfo},
    middleware::AddExtension,
    serve, Router,
};

/// Axum's [axum::serve::Serve] with bounds for a Fantasia instance
pub type Serve = serve::Serve<
    IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
    AddExtension<Router, ConnectInfo<SocketAddr>>,
>;
