// Feature detection and configuration
//
// This module provides runtime feature detection for opt-in TUI improvements.
// All features are backward-compatible and non-breaking.

use std::env;

/// Runtime feature flags for TUI improvements
///
/// Features can be enabled via:
/// 1. Compile-time: `cargo build --features modern-tui`
/// 2. Runtime: `NTK_USE_MODERN_TUI=1 ntk`
/// 3. Config file: `~/.config/nettoolskit/config.toml`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    /// Use modern Ratatui-based TUI instead of standard printf-style UI
    pub use_modern_tui: bool,

    /// Use event-driven architecture instead of polling loop
    pub use_event_driven: bool,

    /// Use frame scheduler for optimized rendering
    pub use_frame_scheduler: bool,

    /// Enable persistent session storage
    pub use_persistent_sessions: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self::detect()
    }
}

impl Features {
    /// Detect features from compile-time flags and environment variables
    ///
    /// Priority order:
    /// 1. Environment variables (highest)
    /// 2. Compile-time features
    /// 3. Default (standard UI)
    pub fn detect() -> Self {
        let compile_time = Self::from_compile_time();
        let env_override = Self::from_env();

        Self {
            use_modern_tui: env_override.use_modern_tui || compile_time.use_modern_tui,
            use_event_driven: env_override.use_event_driven || compile_time.use_event_driven,
            use_frame_scheduler: env_override.use_frame_scheduler
                || compile_time.use_frame_scheduler,
            use_persistent_sessions: env_override.use_persistent_sessions
                || compile_time.use_persistent_sessions,
        }
    }

    /// Detect features from compile-time feature flags
    fn from_compile_time() -> Self {
        Self {
            use_modern_tui: cfg!(feature = "modern-tui"),
            use_event_driven: cfg!(feature = "event-driven"),
            use_frame_scheduler: cfg!(feature = "frame-scheduler"),
            use_persistent_sessions: cfg!(feature = "persistent-sessions"),
        }
    }

    /// Detect features from environment variables
    ///
    /// Environment variables:
    /// - `NTK_USE_MODERN_TUI=1`: Enable modern TUI
    /// - `NTK_USE_EVENT_DRIVEN=1`: Enable event-driven architecture
    /// - `NTK_USE_FRAME_SCHEDULER=1`: Enable frame scheduler
    /// - `NTK_USE_PERSISTENT_SESSIONS=1`: Enable persistent sessions
    fn from_env() -> Self {
        Self {
            use_modern_tui: env_var_is_set("NTK_USE_MODERN_TUI"),
            use_event_driven: env_var_is_set("NTK_USE_EVENT_DRIVEN"),
            use_frame_scheduler: env_var_is_set("NTK_USE_FRAME_SCHEDULER"),
            use_persistent_sessions: env_var_is_set("NTK_USE_PERSISTENT_SESSIONS"),
        }
    }

    /// Check if all modern features are enabled
    pub fn is_full_modern(&self) -> bool {
        self.use_modern_tui && self.use_event_driven && self.use_frame_scheduler
    }

    /// Check if any modern feature is enabled
    pub fn has_any_modern(&self) -> bool {
        self.use_modern_tui
            || self.use_event_driven
            || self.use_frame_scheduler
            || self.use_persistent_sessions
    }

    /// Get human-readable description of enabled features
    pub fn description(&self) -> String {
        let mut features = Vec::new();

        if self.use_modern_tui {
            features.push("modern-tui");
        }

        if self.use_event_driven {
            features.push("event-driven");
        }

        if self.use_frame_scheduler {
            features.push("frame-scheduler");
        }

        if self.use_persistent_sessions {
            features.push("persistent-sessions");
        }

        if features.is_empty() {
            "default".to_string()
        } else {
            features.join(", ")
        }
    }

    /// Print feature status to stdout
    pub fn print_status(&self) {
        tracing::info!("NetToolsKit CLI Features:");
        tracing::info!(
            "  Modern TUI: {}",
            if self.use_modern_tui { "✅" } else { "❌" }
        );
        tracing::info!(
            "  Event-Driven: {}",
            if self.use_event_driven { "✅" } else { "❌" }
        );
        tracing::info!(
            "  Frame Scheduler: {}",
            if self.use_frame_scheduler {
                "✅"
            } else {
                "❌"
            }
        );
        tracing::info!(
            "  Persistent Sessions: {}",
            if self.use_persistent_sessions {
                "✅"
            } else {
                "❌"
            }
        );
    }
}

/// Check if an environment variable is set to a truthy value
fn env_var_is_set(name: &str) -> bool {
    env::var(name)
        .map(|v| {
            let v = v.trim().to_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_features() {
        let features = Features::default();
        // Should default to standard UI
        assert!(!features.use_modern_tui || cfg!(feature = "modern-tui"));
    }

    #[test]
    fn test_env_var_parsing() {
        assert!(env_var_is_set("TEST_VAR") == false); // Not set

        std::env::set_var("TEST_VAR", "1");
        assert!(env_var_is_set("TEST_VAR"));

        std::env::set_var("TEST_VAR", "true");
        assert!(env_var_is_set("TEST_VAR"));

        std::env::set_var("TEST_VAR", "0");
        assert!(!env_var_is_set("TEST_VAR"));

        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_feature_description() {
        let features = Features {
            use_modern_tui: false,
            use_event_driven: false,
            use_frame_scheduler: false,
            use_persistent_sessions: false,
        };
        assert_eq!(features.description(), "default");

        let features = Features {
            use_modern_tui: true,
            use_event_driven: true,
            use_frame_scheduler: false,
            use_persistent_sessions: false,
        };
        assert!(features.description().contains("modern-tui"));
        assert!(features.description().contains("event-driven"));
    }

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
    }
}
