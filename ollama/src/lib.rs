//! Ollama integration for NetToolsKit CLI
//!
//! This crate provides integration with Ollama for local LLM inference:
//! - HTTP client for Ollama API
//! - Model management (list, pull, delete)
//! - Streaming completions and embeddings
//!
//! # Examples
//!
//! ```rust,no_run
//! use nettoolskit_ollama::{OllamaClient, ChatRequest};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = OllamaClient::new("http://localhost:11434");
//!
//!     let request = ChatRequest::new("llama2")
//!         .with_prompt("Hello, Ollama!");
//!
//!     let response = client.chat(request).await?;
//!     println!("Response: {}", response.message);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod models;

pub use client::*;
pub use models::*;
