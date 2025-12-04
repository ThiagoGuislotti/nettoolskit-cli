//! UI Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-ui crate.
//! All module tests are organized to mirror the src/ directory structure.

// Module-specific test suites (hierarchical organization)
mod core;
mod enum_menu_tests;
mod error_tests;
mod integration;
mod interaction;
mod rendering;
