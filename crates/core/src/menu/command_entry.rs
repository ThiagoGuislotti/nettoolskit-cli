//! Command entry trait definition
//!
//! Extension trait for command-like menu entries with slash notation.

use super::MenuEntry;

/// Extension trait for command-like menu entries with slash notation
///
/// Provides default implementations for common patterns like getting
/// command names and slash-prefixed commands. Requires the implementing
/// type to have `strum::IntoStaticStr`.
pub trait CommandEntry: MenuEntry + Into<&'static str> + Copy {
    /// Get the command name (e.g., "help", "manifest")
    ///
    /// Default implementation uses strum's IntoStaticStr conversion.
    fn name(&self) -> &'static str {
        (*self).into()
    }

    /// Get the command with slash prefix (e.g., "/help", "/manifest")
    ///
    /// Default implementation formats the name with a "/" prefix.
    fn slash_static(&self) -> String {
        format!("/{}", self.name())
    }
}
