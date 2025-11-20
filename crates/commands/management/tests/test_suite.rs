//! Management Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-management crate.
//! Test structure mirrors the src/ directory structure:
//! - core/: Command definitions and error handling tests
//! - execution/: Executor and processor tests
//! - handlers/: Command handler tests (apply, help, display)
//! - io_tests: I/O utilities and output formatting tests
//! - integration/: End-to-end integration tests

// Shared test utilities (data folder with test files)
#[path = "data"]
pub mod data {
    // This is just a marker module for the data directory
    // Test files are loaded directly by tests, not as Rust modules
}

// Test modules mirroring src/ structure
mod core;
mod execution;
mod handlers;

// I/O module tests (src/io.rs is a single file, not a directory)
#[path = "io_tests.rs"]
mod io_tests;

// Integration tests
#[path = "integration/integration_tests.rs"]
mod integration_tests;