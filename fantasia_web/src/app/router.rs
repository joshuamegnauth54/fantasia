use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    decompression::DecompressionLayer,
    normalize_path::NormalizePathLayer,
    request_id::MakeRequestUuid,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
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
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // Defaults to true for each enabled compression algo
                // https://github.com/tower-rs/tower-http/blob/6f964b12fd059a87feb8042cc82cdc8af69cb0b8/tower-http/src/compression_utils.rs#L120-L129
                .layer(DecompressionLayer::new())
                .layer(CompressionLayer::new())
                .propagate_x_request_id(),
        )
        .with_state(state)
}
