//! Orchestrator Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-orchestrator crate.
//! Test structure mirrors the src/ directory structure:
//! - models/: MainAction, Command, ExitStatus tests
//! - execution/: Processor and executor tests
//! - error_tests: Error handling tests

// Module tests mirroring src/ structure
#[path = "models/mod.rs"]
mod models;

#[path = "execution/mod.rs"]
mod execution;

#[path = "error_tests.rs"]
mod error_tests;
