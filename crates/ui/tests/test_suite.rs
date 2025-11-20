//! UI Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-ui crate.
//! All module tests are organized to mirror the src/ directory structure.

// Module-specific test suites (hierarchical organization)
mod core;
mod rendering;
mod interaction;

// Error tests (mandatory)
mod error_tests;

// Integration tests
mod integration;
