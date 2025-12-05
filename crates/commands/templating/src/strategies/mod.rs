//! Language-specific template resolution strategies.
//!
//! This module provides strategy pattern for different languages using
//! one-file-per-type organization for maximum maintainability.

pub mod clojure;
pub mod dotnet;
pub mod factory;
pub mod go;
pub mod java;
pub mod language_strategy;
pub mod python;
pub mod rust;

pub use clojure::ClojureStrategy;
pub use dotnet::DotNetStrategy;
pub use factory::{Language, LanguageStrategyFactory};
pub use go::GoStrategy;
pub use java::JavaStrategy;
pub use language_strategy::{LanguageConventions, LanguageStrategy};
pub use python::PythonStrategy;
pub use rust::RustStrategy;