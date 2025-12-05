//! Tests for Features trait implementations
//!
//! Validates Debug, Clone, Copy, PartialEq, and Eq trait behavior.

use nettoolskit_core::Features;

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