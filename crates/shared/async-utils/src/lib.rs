//! Async utilities for NetToolsKit CLI
//!
//! This crate provides async primitives and utilities for handling
//! concurrent operations in the CLI application:
//!
//! - **Cancellation**: Graceful task cancellation with CancellationToken
//! - **Timeouts**: Time-bounded operations with configurable limits
//!
//! # Examples
//!
//! ```rust,no_run
//! use nettoolskit_async_utils::{with_timeout, CancellationToken};
//! use std::time::Duration;
//!
//! async fn example() -> Result<String, Box<dyn std::error::Error>> {
//!     // Execute with timeout
//!     let result = with_timeout(
//!         Duration::from_secs(5),
//!         async { "completed".to_string() }
//!     ).await?;
//!     Ok(result)
//! }
//! ```

pub mod cancellation;
pub mod timeout;

pub use cancellation::*;
pub use timeout::*;
