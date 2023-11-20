use uuid::Uuid;

#[tracing::instrument(level = "debug", fields(
    request_id = %Uuid::new_v4()
))]
pub async fn index() {}
