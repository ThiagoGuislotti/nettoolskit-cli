/// OpenTelemetry utilities for NetToolsKit CLI
///
/// This crate provides comprehensive telemetry, metrics, and structured logging
/// capabilities using OpenTelemetry standards for observability.

pub mod telemetry;
pub mod tracing_setup;

// Re-export main utilities for easy access
pub use telemetry::{Metrics, Timer};
pub use tracing_setup::{
    TracingConfig,
    init_tracing,
    init_tracing_with_config,
    init_tracing_with_filter,
    init_production_tracing,
    init_development_tracing,
    shutdown_tracing
};

// Note: Macros log_operation! and time_operation! are available when using this crate