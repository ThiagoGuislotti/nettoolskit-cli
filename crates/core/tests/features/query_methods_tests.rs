//! Tests for feature query methods
//!
//! Validates `is_full_modern()`, `has_any_modern()`, and `description()`
//! methods for querying feature configuration state.

use nettoolskit_core::Features;
use std::env;
use super::test_helpers::{ENV_LOCK, clear_feature_env_vars};

#[test]
fn test_is_full_modern_with_all_features() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");
    env::set_var("NTK_USE_EVENT_DRIVEN", "1");
    env::set_var("NTK_USE_FRAME_SCHEDULER", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.is_full_modern());
    clear_feature_env_vars();
}

#[test]
fn test_is_full_modern_with_partial_features() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");
    env::set_var("NTK_USE_EVENT_DRIVEN", "1");

    // Act
    let _features = Features::detect();

    // Assert
    #[cfg(not(feature = "frame-scheduler"))]
    assert!(!_features.is_full_modern());
    clear_feature_env_vars();
}

#[test]
fn test_has_any_modern_with_one_feature() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.has_any_modern());
    clear_feature_env_vars();
}

#[test]
fn test_has_any_modern_with_no_features() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let _features = Features::detect();

    // Assert
    #[cfg(not(any(
        feature = "modern-tui",
        feature = "event-driven",
        feature = "frame-scheduler",
        feature = "persistent-sessions"
    )))]
    assert!(!_features.has_any_modern());
}

#[test]
fn test_description_with_single_feature() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");

    // Act
    let features = Features::detect();
    let desc = features.description();

    // Assert
    assert!(desc.contains("modern-tui"));
    clear_feature_env_vars();
}

#[test]
fn test_description_with_multiple_features() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");
    env::set_var("NTK_USE_EVENT_DRIVEN", "1");

    // Act
    let features = Features::detect();
    let desc = features.description();

    // Assert
    assert!(desc.contains("modern-tui"));
    assert!(desc.contains("event-driven"));
    clear_feature_env_vars();
}

#[test]
fn test_all_features_disabled() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(not(any(
        feature = "modern-tui",
        feature = "event-driven",
        feature = "frame-scheduler",
        feature = "persistent-sessions"
    )))]
    {
        assert!(!features.use_modern_tui);
        assert!(!features.use_event_driven);
        assert!(!features.use_frame_scheduler);
        assert!(!features.use_persistent_sessions);
        assert!(!features.is_full_modern());
        assert!(!features.has_any_modern());
    }

    #[cfg(any(
        feature = "modern-tui",
        feature = "event-driven",
        feature = "frame-scheduler",
        feature = "persistent-sessions"
    ))]
    {
        let _ = features; // Suppress unused warning when features are enabled
    }
}
