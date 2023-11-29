use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    routes::{health_check, index},
    state::State,
}; //sql_temp};

pub fn bind_routes(state: State) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
        .with_state(state)
}
