//! Template translation command processor
//!
//! Translates templates from one language strategy to another
//! (e.g., C# → TypeScript, Python → Rust)
//!
//! # Architecture
//!
//! The translation process follows these steps:
//! 1. Parse source and target languages using Language::parse
//! 2. Load template from specified path
//! 3. Parse template using source language strategy
//! 4. Convert intermediate representation to target language
//! 5. Render output using target language strategy

pub mod core;
pub mod handlers;

// Re-exports
pub use core::{TranslateError, TranslateRequest};
pub use handlers::translate::handle_translate;
