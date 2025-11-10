//! Ollama integration for NetToolsKit CLI
//!
//! This crate provides integration with Ollama for local LLM inference:
//! - HTTP client for Ollama API
//! - Model management (list, pull, delete)
//! - Streaming completions and embeddings
//!
//! # Examples
//!
//! ```rust,ignore
//! use nettoolskit_ollama::OllamaClient;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::new(Some("http://localhost:11434".to_string()));
//!     // Use client methods here
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod models;

pub use client::*;
pub use models::*;
