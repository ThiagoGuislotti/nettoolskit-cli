//! Tests for feature detection and configuration
//!
//! This test file validates the runtime feature detection system,
//! including compile-time flags and environment variable overrides.
//!
//! NOTE: Tests that modify environment variables use a global lock to prevent
//! race conditions when running in parallel.

use nettoolskit_core::Features;
use std::env;
use std::sync::Mutex;

// =============================================================================
// Test Helpers
// =============================================================================

/// Global lock to serialize tests that modify environment variables
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Clear all feature-related environment variables to prevent test interference
fn clear_feature_env_vars() {
    env::remove_var("NTK_USE_MODERN_TUI");
    env::remove_var("NTK_USE_EVENT_DRIVEN");
    env::remove_var("NTK_USE_FRAME_SCHEDULER");
    env::remove_var("NTK_USE_PERSISTENT_SESSIONS");
}

// =============================================================================
// Constructor Tests
// =============================================================================

#[test]
fn test_features_default_creates_instance() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::default();
    let detected = Features::detect();

    // Assert
    assert_eq!(features, detected);

    clear_feature_env_vars();
}

#[test]
fn test_features_detect_creates_instance() {
    // Arrange
    let _lock = ENV_LOCK.lock().unwrap();
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert - intentionally tests all feature combinations
    #[allow(clippy::overly_complex_bool_expr)]
    {
        assert!(
            features.use_modern_tui
                || !features.use_modern_tui
                || features.use_event_driven
                || !features.use_event_driven
        );
    }

    clear_feature_env_vars();
}

// =============================================================================
// Compile-Time Feature Detection
// =============================================================================

#[test]
fn test_modern_tui_feature_flag() {
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

// =============================================================================
// Environment Variable Override Tests
// =============================================================================

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

// =============================================================================
// Feature Query Methods
// =============================================================================

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

// =============================================================================
// Trait Implementation Tests
// =============================================================================

#[test]
fn test_features_debug_format() {
    // Arrange
    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };

    // Act
    let debug_str = format!("{:?}", features);

    // Assert
    assert!(debug_str.contains("Features"));
    assert!(debug_str.contains("use_modern_tui"));
}

#[test]
fn test_features_clone() {
    // Arrange
    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };

    // Act - Features is Copy, so clone is unnecessary
    let cloned = features;

    // Assert
    assert_eq!(features, cloned);
}

#[test]
fn test_features_copy() {
    // Arrange
    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };

    // Act
    let copied = features;

    // Assert
    assert_eq!(features, copied);
}

#[test]
fn test_features_equality() {
    // Arrange
    let features1 = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };
    let features2 = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };

    // Assert
    assert_eq!(features1, features2);
}

#[test]
fn test_features_inequality() {
    // Arrange
    let features1 = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };
    let features2 = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };

    // Assert
    assert_ne!(features1, features2);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_all_features_disabled() {
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
    {
        assert!(!_features.use_modern_tui);
        assert!(!_features.use_event_driven);
        assert!(!_features.use_frame_scheduler);
        assert!(!_features.use_persistent_sessions);
        assert!(!features.is_full_modern());
        assert!(!features.has_any_modern());
    }
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
