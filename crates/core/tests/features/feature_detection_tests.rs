//! Tests for compile-time feature flag detection
//!
//! Validates that Cargo feature flags correctly enable/disable
//! corresponding runtime feature fields in the Features struct.

use super::test_helpers::{clear_feature_env_vars, ENV_LOCK};
use nettoolskit_core::Features;

#[test]
fn test_modern_tui_feature_flag() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "modern-tui")]
    assert!(features.use_modern_tui);

    #[cfg(not(feature = "modern-tui"))]
    {
        assert!(!features.use_modern_tui);
    }
}

#[test]
fn test_event_driven_feature_flag() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "event-driven")]
    assert!(features.use_event_driven);

    #[cfg(not(feature = "event-driven"))]
    {
        assert!(!features.use_event_driven);
    }
}

#[test]
fn test_frame_scheduler_feature_flag() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "frame-scheduler")]
    assert!(features.use_frame_scheduler);

    #[cfg(not(feature = "frame-scheduler"))]
    {
        assert!(!features.use_frame_scheduler);
    }
}

#[test]
fn test_persistent_sessions_feature_flag() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "persistent-sessions")]
    assert!(features.use_persistent_sessions);

    #[cfg(not(feature = "persistent-sessions"))]
    {
        assert!(!features.use_persistent_sessions);
    }
}
