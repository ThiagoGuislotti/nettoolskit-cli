//! Help Command Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-help crate.
//! Test structure mirrors the src/ directory structure:
//! - handlers/: Handler tests (discovery, display)
//! - models/: Model tests (ManifestInfo)
//! - error_tests: Error handling tests

// Module tests mirroring src/ structure
#[path = "handlers/mod.rs"]
mod handlers;

#[path = "models/mod.rs"]
mod models;

#[path = "error_tests.rs"]
mod error_tests;
