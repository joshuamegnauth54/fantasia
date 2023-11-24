use std::{
    env,
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{de::Error as DeError, Deserialize};
use tracing::{info, trace};

use super::args::Args;

#[derive(Deserialize, Debug)]
pub struct General {
    /// Host to bind for the application (e.g. `localhost`)
    pub host: String,
    /// `host`'s port
    pub port: u16,
    /// Override `.env` path. Defaults to `.env` otherwise.
    pub env_file: Option<PathBuf>,
    /// Postgres server options.
    #[serde(default)]
    pub postgres: Postgres,
}

#[derive(Deserialize, Debug)]
pub struct Postgres {
    /// Superuser account name
    user: String,
    /// Superuser password
    password: SecretString,
    /// Postgres host
    host: String,
    /// Postgres host's port
    port: u16,
    /// Database name
    database: String,
    /// Postgres pool options for [sqlx].
    #[serde(default)]
    options: Option<PgOptions>,
}

#[derive(Deserialize, Debug)]
pub struct PgOptions {}

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
}

impl General {
    /// Load configuration file from `path`.
    #[tracing::instrument(name = "config from TOML")]
    pub fn from_path<P>(path: P) -> Result<General, toml::de::Error>
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
        if let Ok(user) = env::var("POSTGRES_USER").or_else(|_| env::var("PGUSER")) {
            self.postgres.user = user;
        }

        if let Ok(pass) = env::var("POSTGRES_PASSWORD").or_else(|_| env::var("PGPASSWORD")) {
            self.postgres.password = pass.into();
        }

        if let Ok(host) = env::var("PGHOST") {
            self.postgres.host = host;
        }

        if let Some(port) = env::var("PGPORT").ok().and_then(|port| port.parse().ok()) {
            self.postgres.port = port;
        }

        if let Ok(database) = env::var("POSTGRES_DB").or_else(|_| env::var("PGDATABASE")) {
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
