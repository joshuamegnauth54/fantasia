use std::{
    env,
    fmt::{self, Debug},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use fantasia_web::PgPoolOptions;
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{de::Error as DeError, Deserialize};
use tracing::{info, trace};

use super::args::Args;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Server options for the app as a whole.
    #[serde(default)]
    pub fantasia: Application,
    /// Postgres server options.
    #[serde(default)]
    pub postgres: Postgres,
}

/// General application options, such as the socket address for the server.
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Application {
    /// Host to bind for the application (e.g. `localhost`)
    pub host: String,
    /// `host`'s port
    pub port: u16,
    /// Override `.env` path. Defaults to `.env` otherwise.
    pub env_file: Option<PathBuf>,
}

/// Postgres connection options
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Postgres {
    /// Superuser account name
    pub user: String,
    /// Superuser password
    pub password: SecretString,
    /// Postgres host
    pub host: String,
    /// Postgres host's port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Postgres pool options for [sqlx].
    #[serde(with = "super::pool_options", default)]
    pub options: Option<PgPoolOptions>,
}

/// View into the parameters used to build the Postgres database URL.
pub struct DatabaseUrlView<'s> {
    pub user: &'s str,
    pub password: &'s SecretString,
    pub host: &'s str,
    pub port: u16,
    pub database: &'s str,
    pub url: SecretString,
}

impl ExposeSecret<String> for DatabaseUrlView<'_> {
    fn expose_secret(&self) -> &String {
        self.url.expose_secret()
    }
}

impl Debug for DatabaseUrlView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "postgres://{}:{:?}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

impl Default for Application {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 8000,
            env_file: None,
        }
    }
}

impl Default for Postgres {
    fn default() -> Self {
        Self {
            user: "postgres".into(),
            password: SecretString::new("postgres".into()),
            host: "localhost".into(),
            port: 5432,
            database: "pgdb".into(),
            options: None,
        }
    }
}

impl Postgres {
    /// Create a syntactically valid Postgres database URL.
    ///
    /// Syntactically valid only means that the URL is properly formatted. The content may still be
    /// wrong.
    ///
    /// Acknowledgement: I more or less copied this from Zero to Production.
    pub fn database_url(&self) -> SecretString {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        ))
    }

    pub fn database_url_view(&self) -> DatabaseUrlView<'_> {
        DatabaseUrlView {
            user: &self.user,
            password: &self.password,
            host: &self.host,
            port: self.port,
            database: &self.database,
            url: self.database_url(),
        }
    }
}

impl Config {
    /// Load configuration file from `path`.
    #[tracing::instrument(name = "config from TOML")]
    pub fn from_path<P>(path: P) -> Result<Config, toml::de::Error>
    where
        P: AsRef<Path> + Debug,
    {
        // Load the ostensible TOML file at `path` into a String for the `toml` crate.
        let mut config_reader = File::open(path)
            .map_err(DeError::custom)
            .map(BufReader::new)?;

        let mut config = String::new();
        let amount_read = config_reader
            .read_to_string(&mut config)
            .map_err(DeError::custom)?;
        trace!("Read {amount_read} bytes");

        toml::from_str(&config)
    }

    /// Update configurations with CLI options and Postgres environmental variables.
    ///
    /// CLI options override env vars which in turn override the config file.
    #[tracing::instrument(skip(self))]
    pub fn augment(&mut self, args: Args) {
        // Override loaded settings with CLI options and env vars

        if let Some(host) = args.host {
            self.fantasia.host = host;
        }

        if let Some(port) = args.port {
            self.fantasia.port = port;
        }

        if let Some(user) = args
            .pguser
            .or_else(|| env::var("POSTGRES_USER").ok())
            .or_else(|| env::var("PGUSER").ok())
        {
            self.postgres.user = user;
        }

        if let Some(pass) = args
            .pgpassword
            .or_else(|| env::var("POSTGRES_PASSWORD").ok().map(SecretString::new))
            .or_else(|| env::var("PGPASSWORD").ok().map(SecretString::new))
        {
            self.postgres.password = pass;
        }

        if let Some(host) = args.pghost.or_else(|| env::var("PGHOST").ok()) {
            self.postgres.host = host;
        }

        if let Some(port) = args
            .pgport
            .or_else(|| env::var("PGPORT").ok().and_then(|port| port.parse().ok()))
        {
            self.postgres.port = port;
        }

        if let Some(database) = args
            .pgdatabase
            .or_else(|| env::var("POSTGRES_DB").ok())
            .or_else(|| env::var("PGDATABASE").ok())
        {
            self.postgres.database = database;
        }
    }
}

/// Load environment from a file or .env.
///
/// # Variables
/// | Variables                         | Description       |
/// | ---                               | ---               |
/// | `POSTGRES_USER` `PGUSER`          | Superuser         |
/// | `POSTGRES_PASSWORD` `PGPASSWORD`  | Superuser password |
/// | `PGHOST`                          | Postgres host     |
/// | `PGPORT`                          | Port for host     |
/// | `POSTGRES_DB` `PGDATABASE`        | Database name     |
///
/// # Precedence
/// From highest priority to lowest:
/// * CLI arguments
/// * Environmental variables
/// * Config file
///
#[tracing::instrument]
pub fn dotenv(env_file: Option<&Path>) -> Result<(), dotenvy::Error> {
    match env_file {
        Some(path) => {
            // Fail here if the config provides a path to avoid surprises.
            // It's better to let the app runner know that the path failed.
            dotenvy::from_path(path)?;
            info!(
                "Environment loaded from overridden path: {}",
                path.display()
            );
        }
        None => {
            if let Ok(path) = dotenvy::dotenv() {
                info!("Environment loaded from default path: {}", path.display());
            } else {
                info!("Not using .env");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use secrecy::ExposeSecret;
    use test_log::test;

    use super::{dotenv, Application, Config, Postgres};
    use crate::args::Args;

    #[test]
    fn dotenv_example_env_succeeds() -> Result<(), dotenvy::Error> {
        let path = format!("{}/dev.env", env!("CARGO_MANIFEST_DIR"));
        let path = Path::new(&path);

        dotenv(Some(path))
    }

    #[test]
    fn parse_incomplete_conf_succeeds() -> Result<(), toml::de::Error> {
        let path = format!("{}/fantasia_small.toml", env!("CARGO_MANIFEST_DIR"));
        Config::from_path(path).map(|_| ())
    }

    #[test]
    fn parse_complete_conf_succeeds() -> Result<(), toml::de::Error> {
        let path = format!("{}/fantasia_full.toml", env!("CARGO_MANIFEST_DIR"));
        Config::from_path(path).map(|_| ())
    }

    #[test]
    fn augmenting_config_succeeds() {
        let mut config = Config::default();
        let args = Args {
            conf: None,
            host: None,
            port: Some(666),
            pguser: Some("NotJosh".into()),
            pgpassword: Some("gaben".to_string().into()),
            pghost: None,
            pgport: None,
            pgdatabase: None,
        };

        config.augment(args);
        let expected = Config {
            fantasia: Application {
                port: 666,
                ..Default::default()
            },
            postgres: Postgres {
                user: "NotJosh".into(),
                password: "gaben".to_string().into(),
                ..Default::default()
            },
        };

        assert_eq!(expected.fantasia, config.fantasia);
        assert_eq!(expected.postgres.user, config.postgres.user);
        assert_eq!(
            expected.postgres.password.expose_secret(),
            config.postgres.password.expose_secret()
        );
        assert_eq!(expected.postgres.host, config.postgres.host);
        assert_eq!(expected.postgres.database, config.postgres.database);
        assert_eq!(
            expected.postgres.options.is_none(),
            config.postgres.options.is_none()
        );
    }
}
