use axum::http::StatusCode;

pub async fn fallback_404() -> StatusCode {
    StatusCode::NOT_FOUND
}
