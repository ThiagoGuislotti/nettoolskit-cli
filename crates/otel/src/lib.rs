//! Telemetry, metrics, and structured logging for NetToolsKit CLI
//!
//! This crate provides lightweight, in-process observability utilities:
//!
//! - **Structured logging** via `tracing` + `tracing-subscriber` (env-filter, compact/pretty)
//! - **Metrics** — thread-safe counters, gauges, and timing histograms (custom, in-process)
//! - **Timer** — RAII-based operation timing with auto-record on drop
//!
//! # Design Decision
//!
//! This crate intentionally uses a custom, zero-dependency metrics implementation
//! rather than the full OpenTelemetry SDK. For a CLI tool, the overhead of an OTLP
//! exporter, batched spans, and an external collector is unnecessary. The custom
//! `Metrics`/`Timer` approach provides:
//!
//! - Zero runtime overhead when metrics are not queried
//! - No external collector or network dependency
//! - Simple, predictable behavior for short-lived CLI processes
//!
//! If future requirements demand distributed tracing or OTLP export, the
//! `tracing-opentelemetry` bridge can be added as a layer without changing
//! the existing `Metrics`/`Timer` API.

/// In-process metrics and timers.
pub mod telemetry;
/// Tracing subscriber configuration and initialization.
pub mod tracing_setup;

/// Re-exported metrics and timer types.
pub use telemetry::{Metrics, Timer};
/// Re-exported tracing initialization functions and configuration.
pub use tracing_setup::{
    init_development_tracing, init_production_tracing, init_tracing, init_tracing_with_config,
    init_tracing_with_filter, shutdown_tracing, TracingConfig,
};

// Note: Macros log_operation! and time_operation! are available when using this crate
