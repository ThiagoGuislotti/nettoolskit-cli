//! Regression tests for NetToolsKit CLI
//!
//! These tests ensure that new TUI improvements don't break existing functionality.
//! All tests must pass before merging any TUI-related changes.

use nettoolskit_cli::ExitStatus;
use nettoolskit_core::Features;

/// Test that feature detection works correctly
#[test]
fn test_feature_detection() {
    let features = Features::detect();

    // Should always be able to detect features without panic
    let desc = features.description();
    assert!(!desc.is_empty());

    // Default should be standard UI (unless compiled with modern-tui feature)
    if !cfg!(feature = "modern-tui") {
        assert!(!features.use_modern_tui, "Standard UI should be default");
    }
}

/// Test that Features can be created from default
#[test]
fn test_features_default() {
    let features = Features::default();

    // Should not panic
    assert!(features.description().len() > 0);
}

/// Test feature description formatting
#[test]
fn test_feature_description_formatting() {
    let features = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };

    let desc = features.description();
    assert_eq!(desc, "default");

    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };

    let desc = features.description();
    assert!(desc.contains("modern-tui"));
}

/// Test that environment variable overrides work
#[test]
fn test_env_var_override() {
    // Set env var
    std::env::set_var("NTK_USE_MODERN_TUI", "1");

    let features = Features::detect();

    // Should be enabled via env var
    assert!(
        features.use_modern_tui,
        "Modern TUI should be enabled via env var"
    );

    // Cleanup
    std::env::remove_var("NTK_USE_MODERN_TUI");
}

/// Test various environment variable formats
#[test]
fn test_env_var_formats() {
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

    std::env::remove_var("NTK_TEST_VAR");
}

/// Test that ExitStatus conversion works
#[test]
fn test_exit_status_conversion() {
    let status = ExitStatus::Success;
    let code: i32 = status.into();
    assert_eq!(code, 0);

    let status = ExitStatus::Error;
    let code: i32 = status.into();
    assert_eq!(code, 1);

    let status = ExitStatus::Interrupted;
    let code: i32 = status.into();
    assert_eq!(code, 130);
}

/// Test that Features::is_full_modern works correctly
#[test]
fn test_is_full_modern() {
    let features = Features {
        use_modern_tui: true,
        use_event_driven: true,
        use_frame_scheduler: true,
        use_persistent_sessions: false,
    };
    assert!(features.is_full_modern());

    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    assert!(!features.is_full_modern());

    let features = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    assert!(!features.is_full_modern());
}

/// Test that Features::has_any_modern works correctly
#[test]
fn test_has_any_modern() {
    let features = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    assert!(!features.has_any_modern());

    let features = Features {
        use_modern_tui: true,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: false,
    };
    assert!(features.has_any_modern());

    let features = Features {
        use_modern_tui: false,
        use_event_driven: false,
        use_frame_scheduler: false,
        use_persistent_sessions: true,
    };
    assert!(features.has_any_modern());
}

/// Test that multiple env vars can be set simultaneously
#[test]
fn test_multiple_env_vars() {
    std::env::set_var("NTK_USE_MODERN_TUI", "1");
    std::env::set_var("NTK_USE_EVENT_DRIVEN", "1");

    let features = Features::detect();

    assert!(features.use_modern_tui);
    assert!(features.use_event_driven);

    // Cleanup
    std::env::remove_var("NTK_USE_MODERN_TUI");
    std::env::remove_var("NTK_USE_EVENT_DRIVEN");
}

/// Integration test: Verify that feature detection doesn't panic with any combination
#[test]
fn test_all_feature_combinations() {
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

    for (modern, event, frame, session) in combinations {
        let features = Features {
            use_modern_tui: modern,
            use_event_driven: event,
            use_frame_scheduler: frame,
            use_persistent_sessions: session,
        };

        // Should not panic
        let _ = features.description();
        let _ = features.is_full_modern();
        let _ = features.has_any_modern();
    }
}

/// Test that feature detection is consistent across multiple calls
#[test]
fn test_feature_detection_consistency() {
    let features1 = Features::detect();
    let features2 = Features::detect();

    assert_eq!(features1.use_modern_tui, features2.use_modern_tui);
    assert_eq!(features1.use_event_driven, features2.use_event_driven);
    assert_eq!(features1.use_frame_scheduler, features2.use_frame_scheduler);
    assert_eq!(
        features1.use_persistent_sessions,
        features2.use_persistent_sessions
    );
}
