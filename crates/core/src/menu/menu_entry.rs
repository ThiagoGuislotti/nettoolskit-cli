//! Menu entry trait definition
//!
//! Defines the interface for items that can be displayed in UI menus.

/// Trait for items that can be displayed in UI menus
///
/// This trait defines the interface for items that can be shown in menus,
/// allowing the UI layer to remain decoupled from specific domain types.
pub trait MenuEntry {
    /// Get the label/identifier for this menu entry (e.g., "/list")
    fn label(&self) -> &str;

    /// Get the description for this menu entry
    fn description(&self) -> &str;
}
