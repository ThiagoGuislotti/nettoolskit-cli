//! Test helpers and utilities for feature testing
//!
//! Provides shared test infrastructure including environment
//! variable management and synchronization for parallel test runs.

use std::env;
use std::sync::Mutex;

/// Global lock to serialize tests that modify environment variables
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Clear all feature-related environment variables to prevent test interference
pub fn clear_feature_env_vars() {
    env::remove_var("NTK_USE_MODERN_TUI");
    env::remove_var("NTK_USE_EVENT_DRIVEN");
    env::remove_var("NTK_USE_FRAME_SCHEDULER");
    env::remove_var("NTK_USE_PERSISTENT_SESSIONS");
}