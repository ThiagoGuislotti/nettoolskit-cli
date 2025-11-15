//! NetToolsKit Core - Integration Tests
//!
//! This module aggregates all integration tests for the core crate.
//! Tests are organized by functionality to mirror the src/ structure.

// Module-specific test suites
mod async_utils;
mod config;
mod error_tests;
mod features;
mod file_search;
mod string_utils;

// Cross-module integration tests
mod integration_tests;