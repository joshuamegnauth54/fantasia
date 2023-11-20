use axum::extract::FromRef;
use sqlx::PgPool;

/// Complete app state.
#[derive(Clone)]
pub struct State {
    pub pool: PgPool,
}

/// Database app state.
#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl FromRef<State> for Database {
    fn from_ref(input: &State) -> Self {
        Self {
            pool: input.pool.clone(),
        }
    }
}
