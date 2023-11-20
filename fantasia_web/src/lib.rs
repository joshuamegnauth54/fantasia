pub mod app;
pub mod routes;
pub mod state;

// Reexports
pub use sqlx::{postgres::PgPoolOptions, PgPool};

use axum::{routing::IntoMakeService, Router};
use hyper::server::conn::AddrIncoming;

/// Hyper [hyper::Server] with bounds for a Fantasia instance
pub type Server = axum::Server<AddrIncoming, IntoMakeService<Router>>;
