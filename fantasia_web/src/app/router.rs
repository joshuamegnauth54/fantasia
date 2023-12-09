use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    decompression::DecompressionLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse},
    ServiceBuilderExt
};

use crate::{
    routes::{fallback_404, health_check, index},
    state::State,
}; //sql_temp};

pub fn bind_routes(state: State) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health_check", get(health_check))
        .fallback(fallback_404)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)).on_response(DefaultOnResponse::new().include_headers(true)))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(DecompressionLayer::new())
                .layer(CompressionLayer::new())
                .propagate_x_request_id(),
        )
        .with_state(state)
}
