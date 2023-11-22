use anyhow::Result;

pub(crate) fn logging() -> Result<()> {
    tracing_subscriber::fmt::init();

    Ok(())
}
