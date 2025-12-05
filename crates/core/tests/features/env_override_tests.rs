//! Tests for environment variable override behavior
//!
//! Validates that NTK_USE_* environment variables correctly override
//! compile-time feature flags at runtime.

use nettoolskit_core::Features;
use std::env;
use super::test_helpers::{ENV_LOCK, clear_feature_env_vars};

#[test]
fn test_env_var_enables_modern_tui() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_modern_tui);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_enables_event_driven() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_EVENT_DRIVEN", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_event_driven);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_enables_frame_scheduler() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_FRAME_SCHEDULER", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_frame_scheduler);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_enables_persistent_sessions() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_PERSISTENT_SESSIONS", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_persistent_sessions);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_with_zero_does_not_enable() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "0");

    // Act
    let _features = Features::detect();

    // Assert
    #[cfg(not(feature = "modern-tui"))]
    assert!(!_features.use_modern_tui);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_with_empty_string_does_not_enable() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "");

    // Act
    let _features = Features::detect();

    // Assert
    #[cfg(not(feature = "modern-tui"))]
    assert!(!_features.use_modern_tui);
    clear_feature_env_vars();
}

#[test]
fn test_multiple_env_vars_all_enabled() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");
    env::set_var("NTK_USE_EVENT_DRIVEN", "1");
    env::set_var("NTK_USE_FRAME_SCHEDULER", "1");
    env::set_var("NTK_USE_PERSISTENT_SESSIONS", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_modern_tui);
    assert!(features.use_event_driven);
    assert!(features.use_frame_scheduler);
    assert!(features.use_persistent_sessions);
    clear_feature_env_vars();
}

#[test]
fn test_env_var_priority_over_compile_time() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();
    env::set_var("NTK_USE_MODERN_TUI", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_modern_tui);
    clear_feature_env_vars();
}