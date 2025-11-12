use anyhow::Result;
use std::env;
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Tracing configuration for different environments
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub verbose: bool,
    pub json_format: bool,
    pub with_file: bool,
    pub with_line_numbers: bool,
    pub service_name: String,
    pub service_version: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            json_format: false,
            with_file: false,
            with_line_numbers: true,
            service_name: "nettoolskit-cli".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Initialize comprehensive tracing for NetToolsKit CLI with structured logging
pub fn init_tracing(verbose: bool) -> Result<()> {
    let config = TracingConfig {
        verbose,
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Initialize tracing with full configuration
pub fn init_tracing_with_config(config: TracingConfig) -> Result<()> {
    // Determine log level based on configuration and environment
    let filter = create_env_filter(&config)?;

    info!(
        service_name = %config.service_name,
        service_version = %config.service_version,
        verbose = config.verbose,
        "Initializing structured logging and tracing"
    );

    // Create the base registry
    let registry = tracing_subscriber::registry().with(filter);

    // Configure format layer based on preferences
    if config.json_format {
        warn!("JSON format requested but not available in current setup, using pretty format");
        let pretty_layer = fmt::layer()
            .with_target(true)
            .with_file(config.with_file)
            .with_line_number(config.with_line_numbers)
            .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
            .with_writer(ui_writer::UiMakeWriter)
            .pretty();

        registry.with(pretty_layer).try_init()?;
    } else {
        let compact_layer = fmt::layer()
            .with_target(false)
            .with_file(config.with_file)
            .with_line_number(config.with_line_numbers)
            .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
            .with_writer(ui_writer::UiMakeWriter)
            .compact();

        registry.with(compact_layer).try_init()?;
    }

    info!(
        config = ?config,
        "Tracing initialized successfully"
    );

    Ok(())
}

/// Initialize tracing with custom filter string
pub fn init_tracing_with_filter(filter: &str) -> Result<()> {
    let filter = EnvFilter::try_new(filter)?;

    info!(
        filter = %filter,
        "Initializing tracing with custom filter"
    );

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
                .with_writer(ui_writer::UiMakeWriter),
        )
        .try_init()?;

    Ok(())
}

/// Create environment filter based on configuration
fn create_env_filter(config: &TracingConfig) -> Result<EnvFilter> {
    // Check for environment variable first
    if let Ok(env_filter) = env::var("RUST_LOG") {
        info!(filter = %env_filter, "Using RUST_LOG environment filter");
        return Ok(EnvFilter::try_new(env_filter)?);
    }

    // Create filter based on verbosity level
    let default_filter = if config.verbose {
        "nettoolskit=debug,nettoolskit_cli=debug,nettoolskit_commands=debug,nettoolskit_ui=debug,info"
    } else {
        "nettoolskit=info,warn"
    };

    info!(filter = %default_filter, "Using default filter configuration");
    Ok(EnvFilter::try_new(default_filter)?)
}

/// Initialize minimal tracing for production
pub fn init_production_tracing() -> Result<()> {
    let config = TracingConfig {
        verbose: false,
        json_format: true,
        with_file: false,
        with_line_numbers: false,
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Initialize development tracing with full details
pub fn init_development_tracing() -> Result<()> {
    let config = TracingConfig {
        verbose: true,
        json_format: false,
        with_file: true,
        with_line_numbers: true,
        ..Default::default()
    };

    init_tracing_with_config(config)
}

/// Gracefully shutdown tracing
pub fn shutdown_tracing() {
    warn!("Shutting down tracing subsystem");
    // Note: tracing_subscriber doesn't require explicit shutdown,
    // but this provides a clean shutdown hook for future extensions
}

mod ui_writer {
    use std::io::{self, Write};

    use nettoolskit_ui::append_footer_log;
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    pub struct UiMakeWriter;

    impl<'a> MakeWriter<'a> for UiMakeWriter {
        type Writer = UiWriter;

        fn make_writer(&'a self) -> Self::Writer {
            UiWriter::default()
        }
    }

    #[derive(Default)]
    pub struct UiWriter {
        buffer: Vec<u8>,
    }

    impl UiWriter {
        fn emit_line(&self, bytes: &[u8]) -> io::Result<()> {
            if bytes.is_empty() {
                return Ok(());
            }

            let mut text = String::from_utf8_lossy(bytes).to_string();
            while text.ends_with('\n') || text.ends_with('\r') {
                text.pop();
            }

            if text.trim().is_empty() {
                return Ok(());
            }

            append_footer_log(&text)
        }
    }

    impl Write for UiWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.extend_from_slice(buf);
            while let Some(position) = self.buffer.iter().position(|&byte| byte == b'\n') {
                let chunk: Vec<u8> = self.buffer.drain(..=position).collect();
                self.emit_line(&chunk)?;
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if !self.buffer.is_empty() {
                let chunk = self.buffer.split_off(0);
                self.emit_line(&chunk)?;
            }
            Ok(())
        }
    }
}
