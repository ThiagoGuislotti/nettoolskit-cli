//! Telemetry, metrics, and structured logging for NetToolsKit CLI
//!
//! This crate provides observability utilities for the CLI:
//!
//! - **Structured logging** via `tracing` + `tracing-subscriber` (env-filter, compact/pretty)
//! - **OpenTelemetry traces** via `tracing-opentelemetry` + OTLP exporter (optional, env-driven)
//! - **Metrics** — thread-safe counters, gauges, and timing histograms (custom, in-process),
//!   with optional OTLP metric mirror when exporter env vars are configured
//! - **Timer** — RAII-based operation timing with auto-record on drop
//! - **Correlation IDs** — lightweight execution/command identifiers for log correlation
//!
//! # Design Decision
//!
//! The crate now supports OpenTelemetry trace export when configured through
//! `OTEL_EXPORTER_OTLP_*` (or `NTK_OTLP_*`) environment variables.
//! Metrics continue to be recorded in-process via the custom `Metrics`/`Timer`
//! implementation and can also be mirrored to OTLP when metrics endpoints are configured.
//!
//! This hybrid model provides:
//!
//! - Optional distributed tracing export for collector-backed environments
//! - Optional distributed metrics export for collector-backed environments
//! - Zero network dependency when OTLP endpoint variables are not configured
//! - Simple, predictable local metrics API for short-lived CLI processes

/// Correlation id helpers for tracing context.
pub mod correlation;
/// In-process metrics and timers.
pub mod telemetry;
/// Tracing subscriber configuration and initialization.
pub mod tracing_setup;

/// Re-exported correlation id generator.
pub use correlation::next_correlation_id;
/// Re-exported metrics and timer types.
pub use telemetry::{Metrics, Timer};
/// Re-exported tracing initialization functions and configuration.
pub use tracing_setup::{
    init_development_tracing, init_production_tracing, init_tracing, init_tracing_with_config,
    init_tracing_with_filter, shutdown_tracing, TracingConfig,
};

// Note: Macros log_operation! and time_operation! are available when using this crate
