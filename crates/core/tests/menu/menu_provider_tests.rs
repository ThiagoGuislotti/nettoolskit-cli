//! `MenuProvider` trait tests
//!
//! Tests for menu providers with `all()` method.

use nettoolskit_core::{MenuEntry, MenuProvider};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
enum TestMenu {
    Option1,
    Option2,
    Option3,
}

impl std::fmt::Display for TestMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl MenuEntry for TestMenu {
    fn label(&self) -> &str {
        match self {
            Self::Option1 => "option1",
            Self::Option2 => "option2",
            Self::Option3 => "option3",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Option1 => "First option",
            Self::Option2 => "Second option",
            Self::Option3 => "Third option",
        }
    }
}

impl MenuProvider for TestMenu {
    fn menu_items() -> Vec<String> {
        Self::iter()
            .map(|item| format!("{} - {}", item.label(), item.description()))
            .collect()
    }

    fn all_variants() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[test]
fn test_menu_provider_all_variants() {
    let items = TestMenu::all_variants();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], TestMenu::Option1);
    assert_eq!(items[1], TestMenu::Option2);
    assert_eq!(items[2], TestMenu::Option3);
}

#[test]
fn test_menu_provider_menu_items() {
    let items = TestMenu::menu_items();
    assert_eq!(items.len(), 3);
    assert!(items[0].contains("option1"));
    assert!(items[0].contains("First option"));
}

#[test]
fn test_menu_provider_with_menu_entry() {
    let items = TestMenu::all_variants();
    for item in items {
        assert!(!item.label().is_empty());
        assert!(!item.description().is_empty());
    }
}
