//! Tests for compile-time feature flag detection
//!
//! Validates that Cargo feature flags correctly enable/disable
//! corresponding runtime feature fields in the Features struct.

use nettoolskit_core::Features;
use std::env;

#[test]
fn test_modern_tui_feature_flag() {
    // Arrange
    // (no setup needed)

    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "modern-tui")]
    assert!(features.use_modern_tui);

    #[cfg(not(feature = "modern-tui"))]
    {
        if env::var("NTK_USE_MODERN_TUI").is_err() {
            assert!(!features.use_modern_tui);
        }
    }
}

#[test]
fn test_event_driven_feature_flag() {
    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "event-driven")]
    assert!(features.use_event_driven);

    #[cfg(not(feature = "event-driven"))]
    {
        if env::var("NTK_USE_EVENT_DRIVEN").is_err() {
            assert!(!features.use_event_driven);
        }
    }
}

#[test]
fn test_frame_scheduler_feature_flag() {
    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "frame-scheduler")]
    assert!(features.use_frame_scheduler);

    #[cfg(not(feature = "frame-scheduler"))]
    {
        if env::var("NTK_USE_FRAME_SCHEDULER").is_err() {
            assert!(!features.use_frame_scheduler);
        }
    }
}

#[test]
fn test_persistent_sessions_feature_flag() {
    // Act
    let features = Features::detect();

    // Assert
    #[cfg(feature = "persistent-sessions")]
    assert!(features.use_persistent_sessions);

    #[cfg(not(feature = "persistent-sessions"))]
    {
        if env::var("NTK_USE_PERSISTENT_SESSIONS").is_err() {
            assert!(!features.use_persistent_sessions);
        }
    }
}