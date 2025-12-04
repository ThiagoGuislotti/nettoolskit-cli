//! Otel Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-otel crate.
//! Test structure mirrors the src/ directory structure:
//! - error_tests: Error handling tests
//! - telemetry_tests: Telemetry and tracing tests

// Module tests mirroring src/ structure
#[path = "error_tests.rs"]
mod error_tests;

#[path = "telemetry_tests.rs"]
mod telemetry_tests;
