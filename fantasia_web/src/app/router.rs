use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::{health_check, index}; //sql_temp};

pub fn bind_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
}
