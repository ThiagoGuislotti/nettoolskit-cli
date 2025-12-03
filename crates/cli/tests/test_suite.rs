//! CLI Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-cli crate.
//! Test structure mirrors the src/ directory structure:
//! - display_tests: Display functionality tests (src/display.rs)
//! - events_tests: Event handling tests (src/events.rs)
//! - input_tests: Input processing tests (src/input.rs)
//! - error_tests: Error handling tests
//! - integration/: End-to-end integration tests (CLI + UI + Core)

// Module tests mirroring src/ files
#[path = "display_tests.rs"]
mod display_tests;

#[path = "events_tests.rs"]
mod events_tests;

#[path = "input_tests.rs"]
mod input_tests;

#[path = "error_tests.rs"]
mod error_tests;
