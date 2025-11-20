//! Manifest Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-manifest crate.
//! All module tests are organized to mirror the src/ directory structure.

mod core;
mod execution;
mod parsing;
mod tasks;

// Integration tests in subdirectory
#[path = "integration/integration_tests.rs"]
mod integration_tests;