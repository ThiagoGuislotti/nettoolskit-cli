use anyhow::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing for NetToolsKit CLI
pub fn init_tracing(verbose: bool) -> Result<()> {
    let filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("nettoolskit=debug,info"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .try_init()?;

    Ok(())
}

/// Initialize tracing with custom filter
pub fn init_tracing_with_filter(filter: &str) -> Result<()> {
    let filter = EnvFilter::try_new(filter)?;

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .try_init()?;

    Ok(())
}