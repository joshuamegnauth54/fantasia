use std::path::PathBuf;

use pico_args::Arguments;
use secrecy::SecretString;
use tracing::error;

/// Command line arguments.
#[derive(Debug)]
pub struct Args {
    /// Override config file
    pub conf: Option<PathBuf>,
    /// Override Fantasia host
    pub host: Option<String>,
    /// Override Fantasia port
    pub port: Option<u16>,
    /// Override Postgres superuser
    pub pguser: Option<String>,
    /// Override Postgres superuser password
    pub pgpassword: Option<SecretString>,
    /// Override Postgres host
    pub pghost: Option<String>,
    /// Override Postgres port
    pub pgport: Option<u16>,
    /// Override Postgres database
    pub pgdatabase: Option<String>,
}

impl Args {
    /// Parse CLI arguments.
    ///
    /// This returns an error on spurious arguments.
    #[tracing::instrument]
    pub fn parse_args() -> Result<Self, pico_args::Error> {
        let mut pargs = Arguments::from_env();

        let args = Self {
            conf: pargs.opt_value_from_str("--config")?,
            host: pargs.opt_value_from_str("--host")?,
            port: pargs.opt_value_from_fn("--port", str::parse)?,
            pguser: pargs.opt_value_from_str("--pguser")?,
            pgpassword: pargs.opt_value_from_str("--pgpassword")?,
            pghost: pargs.opt_value_from_str("--pghost")?,
            pgport: pargs.opt_value_from_fn("--pgport", str::parse)?,
            pgdatabase: pargs.opt_value_from_str("--pgdatabase")?,
        };

        // Fail on extra or invalid arguments
        // It's likely that the invoker intended to use an actual option therefore continuing would
        // be surprising behavior.
        let remaining = pargs.finish();
        if !remaining.is_empty() {
            for extra in remaining {
                error!("Invalid argument: {}", extra.to_string_lossy());
            }
            Err(pico_args::Error::ArgumentParsingFailed {
                cause: "Invoked Fantasia with invalid arguments".into(),
            })
        } else {
            Ok(args)
        }
    }
}
