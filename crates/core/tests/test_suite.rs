//! Core Test Suite Entry Point
//!
//! Main test suite aggregator for nettoolskit-core crate.
//! All module tests are organized to mirror the src/ directory structure.

// Module-specific test suites
mod async_utils;
mod error_tests;
mod features;
mod file_search;
mod menu;
#[path = "path-utils/mod.rs"]
mod path_utils;
mod string_utils;