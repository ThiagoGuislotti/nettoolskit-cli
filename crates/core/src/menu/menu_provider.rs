//! Menu provider trait definition
//!
//! Trait for enums that can provide interactive menu functionality.

use super::MenuEntry;

/// Trait for enums that can provide interactive menu functionality
///
/// This trait combines MenuEntry with iteration capabilities to enable
/// generic menu rendering. Any enum that implements this trait can be
/// used with the generic menu system.
///
/// # Requirements
/// - Must implement `MenuEntry` for label/description
/// - Must be `Clone` for menu interaction
/// - Must be `Display` for rendering
pub trait MenuProvider: MenuEntry + Clone + std::fmt::Display {
    /// Generate formatted menu items for display
    ///
    /// Returns a vector of strings in the format "label - description"
    fn menu_items() -> Vec<String>
    where
        Self: Sized;

    /// Get all enum variants as a vector
    fn all_variants() -> Vec<Self>
    where
        Self: Sized;
}
