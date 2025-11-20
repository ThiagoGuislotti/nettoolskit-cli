//! Language-specific template resolution strategies.
//!
//! This module provides strategy pattern for different languages:
//! - `strategy`: Base trait and language-specific implementations
//! - `factory`: Factory for creating language strategies

pub mod factory;
pub mod strategy;

pub use factory::{Language, LanguageStrategyFactory};
pub use strategy::{
    ClojureStrategy, DotNetStrategy, GoStrategy, JavaStrategy, LanguageConventions,
    LanguageStrategy, PythonStrategy, RustStrategy,
};