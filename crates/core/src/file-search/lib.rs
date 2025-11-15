//! File search utilities for NetToolsKit CLI
//!
//! This crate provides efficient file system search capabilities with:
//! - Pattern-based file filtering (gitignore-style)
//! - Async directory traversal
//! - Configurable search constraints (depth, file types, etc.)
//!
//! # Examples
//!
//! ```rust,no_run
//! use nettoolskit_core::file_search::{SearchConfig, search_files};
//!
//! fn example() {
//!     let config = SearchConfig::default();
//!
//!     let files = search_files("./src", &config).unwrap();
//!     println!("Found {} files", files.len());
//! }
//! ```

/// File filtering utilities
pub mod filters;
/// File search implementation
pub mod search;

pub use filters::*;
pub use search::*;
