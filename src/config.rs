use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct General {
    pub host: String,
    pub env_file: Option<PathBuf>,
    pub postgres: Option<Postgres>
}

#[derive(Deserialize, Serialize)]
pub struct Postgres {

}
