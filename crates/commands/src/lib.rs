//! Command processing and execution for NetToolsKit CLI
//!
//! This crate provides the core command processing logic, including:
//! - Command parsing and validation
//! - Async command execution with progress tracking
//! - Template rendering and application
//! - Manifest orchestration
//! - Exit status handling
//!
//! # Architecture
//!
//! Commands are organized into feature-specific modules:
//! - **management**: Command definitions, registry, processor, executor
//! - **manifest**: Manifest-driven code generation
//! - **templating**: Template rendering engine
//! - **translate**: Code translation between languages
//!
//! # Features
//!
//! - **Management**: Centralized command orchestration
//! - **Templating**: Code generation via Handlebars templates
//! - **Manifest**: Manifest-driven workflows
//! - **Translate**: Cross-language code translation

// Re-export management crate (command orchestration)
pub use nettoolskit_management::{
    menu_entries, palette_entries, process_command, process_text, AsyncCommandExecutor, Command,
    CommandHandle, CommandProgress, CommandResult, ExitStatus, ProgressSender,
};

// Re-export internal feature crates
pub use nettoolskit_manifest;
pub use nettoolskit_templating;
pub use nettoolskit_translate;
