//! [PgPoolOptions] remote type for [serde].
//!
//! This type is needed because [serde]'s traits cannot be implemented for foreign types.

use std::time::Duration;

use serde::{Deserialize, Deserializer};

use fantasia_web::PgPoolOptions;

/// Remote type for [PgPoolOptions].
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(remote = "PgPoolOptions", deny_unknown_fields)]
pub struct PoolOptionsDef {
    // after_connect:
    #[serde(getter = "PgPoolOptions::get_test_before_acquire", default)]
    test_before_acquire: bool,
    #[serde(getter = "PgPoolOptions::get_acquire_timeout", default)]
    acquire_timeout: Duration,
    #[serde(getter = "PgPoolOptions::get_min_connections", default)]
    min_connections: u32,
    #[serde(getter = "PgPoolOptions::get_max_connections", default)]
    max_connections: u32,
    #[serde(getter = "PgPoolOptions::get_max_lifetime", default)]
    max_lifetime: Duration,
    #[serde(getter = "PgPoolOptions::get_idle_timeout", default)]
    idle_timeout: Duration,
}

impl Default for PoolOptionsDef {
    fn default() -> Self {
        let defaults = PgPoolOptions::new();
        Self {
            test_before_acquire: defaults.get_test_before_acquire(),
            acquire_timeout: defaults.get_acquire_timeout(),
            min_connections: defaults.get_min_connections(),
            max_connections: defaults.get_max_connections(),
            max_lifetime: defaults
                .get_max_lifetime()
                .expect("`sqlx` defines `PgPoolOptions::max_lifetime` internally"),
            idle_timeout: defaults
                .get_idle_timeout()
                .expect("`sqlx` defines `PgPoolOptions::idle_timeout` internally"),
        }
    }
}

impl From<PoolOptionsDef> for PgPoolOptions {
    fn from(value: PoolOptionsDef) -> Self {
        PgPoolOptions::new()
            .test_before_acquire(value.test_before_acquire)
            .acquire_timeout(value.acquire_timeout)
            .min_connections(value.min_connections)
            .max_connections(value.max_connections)
            .max_lifetime(value.max_lifetime)
            .idle_timeout(value.idle_timeout)
    }
}

#[derive(Deserialize)]
struct Helper(#[serde(with = "PoolOptionsDef")] PgPoolOptions);

/// Deserialize [PgPoolOptions] wrapped in an [Option] with remote derive.
///
/// https://github.com/serde-rs/serde/issues/1301
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PgPoolOptions>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper = Option::deserialize(deserializer)?;
    Ok(helper.map(|Helper(external)| external))
}
