//! Translate Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-translate crate.
//! Test structure mirrors the src/ directory structure:
//! - core/: Error handling tests
//! - handlers/: Translation handler tests

// Test modules mirroring src/ structure
mod core;
mod handlers;