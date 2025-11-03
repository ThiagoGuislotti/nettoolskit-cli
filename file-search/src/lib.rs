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
//! use nettoolskit_file_search::{FileSearchConfig, search_files};
//!
//! async fn example() {
//!     let config = FileSearchConfig::default()
//!         .with_pattern("*.rs")
//!         .max_depth(3);
//!
//!     let files = search_files("./src", &config).await.unwrap();
//!     println!("Found {} files", files.len());
//! }
//! ```

pub mod filters;
pub mod search;

pub use filters::*;
pub use search::*;
