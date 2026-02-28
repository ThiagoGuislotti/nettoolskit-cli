//! Language-specific template resolution strategies.
//!
//! This module provides strategy pattern for different languages using
//! one-file-per-type organization for maximum maintainability.

/// Clojure language strategy.
pub mod clojure;
/// .NET / C# language strategy.
pub mod dotnet;
/// Language strategy factory and `Language` enum.
pub mod factory;
/// Go language strategy.
pub mod go;
/// Java language strategy.
pub mod java;
/// Core `LanguageStrategy` trait and conventions.
pub mod language_strategy;
/// Python language strategy.
pub mod python;
/// Rust language strategy.
pub mod rust;
/// TypeScript language strategy.
pub mod typescript;

/// Clojure strategy re-export.
pub use clojure::ClojureStrategy;
/// .NET strategy re-export.
pub use dotnet::DotNetStrategy;
/// Factory and language enum re-exports.
pub use factory::{Language, LanguageStrategyFactory};
/// Go strategy re-export.
pub use go::GoStrategy;
/// Java strategy re-export.
pub use java::JavaStrategy;
/// Core strategy trait and conventions re-exports.
pub use language_strategy::{LanguageConventions, LanguageStrategy};
/// Python strategy re-export.
pub use python::PythonStrategy;
/// Rust strategy re-export.
pub use rust::RustStrategy;
/// TypeScript strategy re-export.
pub use typescript::TypeScriptStrategy;
