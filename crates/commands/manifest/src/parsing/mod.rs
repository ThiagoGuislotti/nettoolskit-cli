//! YAML manifest parsing.
//!
//! This module provides functionality to parse YAML manifest files
//! into strongly-typed Rust structures.

/// YAML manifest parser implementation.
pub mod parser;

pub use parser::ManifestParser;
