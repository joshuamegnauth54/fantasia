use std::path::PathBuf;

use pico_args::Arguments;
use tracing::error;

#[derive(Debug)]
pub struct Args {
    pub conf: Option<PathBuf>,
}

impl Args {
    #[tracing::instrument]
    pub fn parse_args() -> Result<Self, pico_args::Error> {
        let mut pargs = Arguments::from_env();

        let args = Self {
            conf: pargs.opt_value_from_str("--config")?,
        };

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
