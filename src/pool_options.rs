//! [PgPoolOptions] remote type for [serde].
//!
//! This type is needed because [serde]'s traits cannot be implemented for foreign types.

use std::time::Duration;

use serde::{Deserialize, Deserializer};

use fantasia_web::PgPoolOptions;

/// Remote type for [PgPoolOptions].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(
    remote = "PgPoolOptions",
    default = "PoolOptionsDef::default_pooloptions",
    deny_unknown_fields
)]
pub struct PoolOptionsDef {
    #[serde(getter = "PgPoolOptions::get_test_before_acquire", default)]
    test_before_acquire: bool,
    #[serde(
        getter = "PgPoolOptions::get_acquire_timeout",
        deserialize_with = "deserialize_duration",
        alias = "acquire_timeout_seconds",
        default
    )]
    acquire_timeout: Duration,
    #[serde(getter = "PgPoolOptions::get_min_connections", default)]
    min_connections: u32,
    #[serde(getter = "PgPoolOptions::get_max_connections", default)]
    max_connections: u32,
    #[serde(
        getter = "PgPoolOptions::get_max_lifetime",
        deserialize_with = "deserialize_duration",
        alias = "max_lifetime_seconds",
        default
    )]
    max_lifetime: Duration,
    #[serde(
        getter = "PgPoolOptions::get_idle_timeout",
        deserialize_with = "deserialize_duration",
        alias = "idle_timeout_seconds",
        default
    )]
    idle_timeout: Duration,
}

impl PoolOptionsDef {
    fn default_pooloptions() -> PgPoolOptions {
        Self::default().into()
    }
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

impl From<PgPoolOptions> for PoolOptionsDef {
    fn from(value: PgPoolOptions) -> Self {
        let defaults = PoolOptionsDef::default();

        Self {
            test_before_acquire: value.get_test_before_acquire(),
            acquire_timeout: value.get_acquire_timeout(),
            min_connections: value.get_min_connections(),
            max_connections: value.get_max_connections(),
            max_lifetime: value.get_max_lifetime().unwrap_or(defaults.max_lifetime),
            idle_timeout: value.get_idle_timeout().unwrap_or(defaults.idle_timeout),
        }
    }
}

#[derive(Debug, Deserialize)]
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

// Deserialize [std::time::Duration] from seconds
fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use fantasia_web::PgPoolOptions;
    use serde_test::{assert_de_tokens, Token};

    use super::Helper;
    use super::PoolOptionsDef;

    impl PartialEq for Helper {
        fn eq(&self, other: &Self) -> bool {
            // Only interested in the fields on the delegate type
            let delegate: PoolOptionsDef = self.0.clone().into();
            let delegate_other = other.0.clone().into();

            delegate == delegate_other
        }
    }

    #[test]
    fn deserializing_full_pgpoolopts_succeeds() {
        let defaults = PoolOptionsDef::default();

        assert_de_tokens(
            &Helper(PgPoolOptions::new()),
            &[
                Token::TupleStruct {
                    name: "Helper",
                    len: 1,
                },
                Token::Struct {
                    name: "PoolOptionsDef",
                    len: 6,
                },
                Token::Str("test_before_acquire"),
                Token::Bool(defaults.test_before_acquire),
                Token::Str("acquire_timeout"),
                Token::U64(defaults.acquire_timeout.as_secs()),
                Token::Str("min_connections"),
                Token::U32(defaults.min_connections),
                Token::Str("max_connections"),
                Token::U32(defaults.max_connections),
                Token::Str("max_lifetime"),
                Token::U64(defaults.max_lifetime.as_secs()),
                Token::Str("idle_timeout"),
                Token::U64(defaults.idle_timeout.as_secs()),
                Token::StructEnd,
                Token::TupleStructEnd,
            ],
        )
    }

    #[test]
    fn deserializing_incomplete_pgpoolopts_succeeds() {
        assert_de_tokens(
            &Helper(PgPoolOptions::new()),
            &[
                Token::TupleStruct {
                    name: "Helper",
                    len: 1,
                },
                Token::Struct {
                    name: "PoolOptionsDef",
                    len: 6,
                },
                Token::StructEnd,
                Token::TupleStructEnd,
            ],
        )
    }
}
