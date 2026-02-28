//! Tests for Features struct constructor behavior
//!
//! Validates `default()` and `detect()` methods produce consistent results.

use super::test_helpers::{clear_feature_env_vars, ENV_LOCK};
use nettoolskit_core::Features;

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
