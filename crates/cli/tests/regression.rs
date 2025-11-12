//! Regression tests for NetToolsKit CLI
//!
//! These tests ensure that new TUI improvements don't break existing functionality.
//! All tests must pass before merging any TUI-related changes.

use nettoolskit_commands::ExitStatus;
use nettoolskit_core::Features;
use std::env;

/// Clear all feature-related environment variables to prevent test interference
fn clear_feature_env_vars() {
    env::remove_var("NTK_USE_MODERN_TUI");
    env::remove_var("NTK_USE_EVENT_DRIVEN");
    env::remove_var("NTK_USE_FRAME_SCHEDULER");
    env::remove_var("NTK_USE_PERSISTENT_SESSIONS");
}

// Feature Detection Tests

#[test]
fn test_feature_detection() {
    // Arrange
    clear_feature_env_vars();

    // Act
    let features = Features::detect();

    // Assert
    let desc = features.description();
    assert!(!desc.is_empty());
    if !cfg!(feature = "modern-tui") {
        assert!(!features.use_modern_tui, "Standard UI should be default");
    }
}

#[test]
fn test_features_default() {
    // Arrange & Act
    let features = Features::default();

    // Assert
    assert!(!features.description().is_empty());
}

#[test]
fn test_feature_description_formatting() {
    // Arrange
    let default_features = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    let modern_features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };

    // Act
    let default_desc = default_features.description();
    let modern_desc = modern_features.description();

    // Assert
    assert_eq!(default_desc, "default");
    assert!(modern_desc.contains("modern-tui"));
}

#[test]
fn test_env_var_override() {
    // Arrange
    std::env::set_var("NTK_USE_MODERN_TUI", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(
        features.use_modern_tui,
        "Modern TUI should be enabled via env var"
    );

    // Cleanup
    std::env::remove_var("NTK_USE_MODERN_TUI");
}

#[test]
fn test_env_var_formats() {
    // Arrange
    let test_cases = vec![
        ("1", true),
        ("true", true),
        ("TRUE", true),
        ("yes", true),
        ("YES", true),
        ("on", true),
        ("ON", true),
        ("0", false),
        ("false", false),
        ("no", false),
        ("", false),
    ];

    // Act & Assert
    for (value, expected) in test_cases {
        std::env::set_var("NTK_TEST_VAR", value);
        let is_set = std::env::var("NTK_TEST_VAR")
            .map(|v| {
                let v = v.trim().to_lowercase();
                v == "1" || v == "true" || v == "yes" || v == "on"
            })
            .unwrap_or(false);
        assert_eq!(
            is_set, expected,
            "Failed for value '{}', expected {}",
            value, expected
        );
    }

    // Cleanup
    std::env::remove_var("NTK_TEST_VAR");
}

// Exit Status Tests

#[test]
fn test_exit_status_conversion() {
    // Arrange & Act
    let success_code: i32 = ExitStatus::Success.into();
    let error_code: i32 = ExitStatus::Error.into();
    let interrupted_code: i32 = ExitStatus::Interrupted.into();

    // Assert
    assert_eq!(success_code, 0);
    assert_eq!(error_code, 1);
    assert_eq!(interrupted_code, 130);
}

// Feature Flags Tests

#[test]
fn test_is_full_modern() {
    // Arrange
    let full_modern = Features {
        use_modern_tui: true,
        use_event_driven: true,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };
    let partial_modern = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    let no_modern = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };

    // Act & Assert
    assert!(full_modern.is_full_modern());
    assert!(!partial_modern.is_full_modern());
    assert!(!no_modern.is_full_modern());
}

#[test]
fn test_has_any_modern() {
    // Arrange
    let no_modern = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    let has_tui = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    let has_sessions = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: true,
    };

    // Act & Assert
    assert!(!no_modern.has_any_modern());
    assert!(has_tui.has_any_modern());
    assert!(has_sessions.has_any_modern());
}

#[test]
fn test_multiple_env_vars() {
    // Arrange
    std::env::set_var("NTK_USE_MODERN_TUI", "1");
    std::env::set_var("NTK_USE_EVENT_DRIVEN", "1");

    // Act
    let features = Features::detect();

    // Assert
    assert!(features.use_modern_tui);
    assert!(features.use_event_driven);

    // Cleanup
    std::env::remove_var("NTK_USE_MODERN_TUI");
    std::env::remove_var("NTK_USE_EVENT_DRIVEN");
}

// Integration Tests

#[test]
fn test_all_feature_combinations() {
    // Arrange
    let combinations = vec![
        (false, false, false, false),
        (true, false, false, false),
        (true, true, false, false),
        (true, true, true, false),
        (true, true, true, true),
        (false, true, false, false),
        (false, false, true, false),
        (false, false, false, true),
    ];

    // Act & Assert - Critical: All combinations should work without panic
    for (modern, event, frame, session) in combinations {
        let features = Features {
            use_modern_tui: modern,
            use_event_driven: event,
            use_frame_scheduler: frame,
            use_persistent_sessions: session,
        };
        let _ = features.description();
        let _ = features.is_full_modern();
        let _ = features.has_any_modern();
    }
}

#[test]
fn test_feature_detection_consistency() {
    // Arrange
    clear_feature_env_vars();

    // Act
    let features1 = Features::detect();
    let features2 = Features::detect();

    // Assert
    assert_eq!(features1.use_modern_tui, features2.use_modern_tui);
    assert_eq!(features1.use_event_driven, features2.use_event_driven);
    assert_eq!(features1.use_frame_scheduler, features2.use_frame_scheduler);
    assert_eq!(
        features1.use_persistent_sessions,
        features2.use_persistent_sessions
    );
}
