//! User configuration for NetToolsKit CLI
//!
//! Provides a layered configuration system with the following priority:
//!
//! 1. **CLI arguments** (highest) — e.g. `--verbose`, `--config`
//! 2. **Environment variables** — `NTK_VERBOSE`, `NTK_COLOR`, etc.
//! 3. **Config file** — `~/.ntk/config.toml`
//! 4. **Defaults** (lowest)
//!
//! # Config File Location
//!
//! - Linux/macOS: `~/.config/ntk/config.toml` (XDG) or `~/.ntk/config.toml`
//! - Windows: `%APPDATA%\ntk\config.toml`
//!
//! # Example Config
//!
//! ```toml
//! [general]
//! verbose = false
//! log_level = "info"
//! footer_output = true
//! runtime_mode = "cli"
//! attention_bell = false
//! attention_desktop_notification = false
//! attention_unfocused_only = false
//! predictive_input = true
//! ai_session_retention = 20
//!
//! [display]
//! color = "auto"          # "auto", "always", "never"
//! unicode = "auto"        # "auto", "always", "never"
//!
//! [templates]
//! directory = "~/.ntk/templates"
//!
//! [shell]
//! default_shell = "bash"  # "bash", "zsh", "fish", "powershell"
//! ```

use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::runtime::{resolve_runtime_mode, RuntimeMode};

/// Primary application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AppConfig {
    /// General settings
    pub general: GeneralConfig,

    /// Display settings (colors, Unicode)
    pub display: DisplayConfig,

    /// Template directory settings
    pub templates: TemplateConfig,

    /// Shell preference
    pub shell: ShellConfig,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeneralConfig {
    /// Enable verbose output
    pub verbose: bool,

    /// Log level filter (trace, debug, info, warn, error)
    pub log_level: String,

    /// Enable interactive footer output stream
    pub footer_output: bool,

    /// Runtime mode selection: `cli` or `service`
    pub runtime_mode: RuntimeMode,

    /// Emit terminal attention bell on command failures in interactive mode
    pub attention_bell: bool,

    /// Emit desktop notifications on command failures in interactive mode
    pub attention_desktop_notification: bool,

    /// Only emit attention bell when terminal focus is lost
    pub attention_unfocused_only: bool,

    /// Enable predictive slash-command hints in interactive input
    pub predictive_input: bool,

    /// Number of local AI session snapshots retained on disk
    pub ai_session_retention: usize,
}

/// Display and rendering settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DisplayConfig {
    /// Color mode: auto, always, never
    pub color: ColorMode,

    /// Unicode mode: auto, always, never
    pub unicode: UnicodeMode,
}

/// Template engine settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct TemplateConfig {
    /// Custom template directory path
    pub directory: Option<String>,
}

/// Shell preference settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct ShellConfig {
    /// Preferred shell for completions and scripts
    pub default_shell: Option<String>,
}

/// Color output mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    /// Detect automatically (check `NO_COLOR`, `TERM`, terminal capabilities)
    #[default]
    Auto,
    /// Always emit color codes
    Always,
    /// Never emit color codes
    Never,
}

/// Unicode output mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UnicodeMode {
    /// Detect automatically (check TERM, locale, Windows version)
    #[default]
    Auto,
    /// Always use Unicode characters (box drawing, emojis)
    Always,
    /// Use ASCII-only fallback
    Never,
}

// --- Defaults ---

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            log_level: "info".to_string(),
            footer_output: true,
            runtime_mode: RuntimeMode::Cli,
            attention_bell: false,
            attention_desktop_notification: false,
            attention_unfocused_only: false,
            predictive_input: true,
            ai_session_retention: 20,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: ColorMode::Auto,
            unicode: UnicodeMode::Auto,
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}

impl fmt::Display for UnicodeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}

// --- Config Loading ---

impl AppConfig {
    /// Load configuration with full fallback chain:
    /// config file → environment variables → defaults
    ///
    /// CLI arguments should be applied after calling this.
    #[must_use]
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        config
    }

    /// Load configuration from a specific file path
    ///
    /// # Errors
    ///
    /// Returns `Err` if the file cannot be read or parsed.
    pub fn load_from(path: &Path) -> crate::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;

        info!(path = %path.display(), "Configuration loaded from file");
        Ok(config)
    }

    /// Attempt to load config from the default file location.
    /// Returns `None` if no config file exists (this is normal).
    fn load_from_file() -> Option<Self> {
        let path = Self::default_config_path()?;

        if !path.exists() {
            debug!(path = %path.display(), "No config file found, using defaults");
            return None;
        }

        match Self::load_from(&path) {
            Ok(config) => Some(config),
            Err(e) => {
                warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to load config file, using defaults"
                );
                None
            }
        }
    }

    /// Apply environment variable overrides on top of file/default config
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("NTK_VERBOSE") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.verbose = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_LOG_LEVEL") {
            self.general.log_level = val;
        }

        if let Ok(val) = env::var("NTK_FOOTER_OUTPUT") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.footer_output = parsed;
            }
        }

        self.general.runtime_mode = resolve_runtime_mode(
            self.general.runtime_mode,
            env::var("NTK_RUNTIME_MODE").ok().as_deref(),
        );

        if let Ok(val) = env::var("NTK_ATTENTION_BELL") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.attention_bell = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_ATTENTION_DESKTOP_NOTIFICATION") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.attention_desktop_notification = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_ATTENTION_UNFOCUSED_ONLY") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.attention_unfocused_only = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_PREDICTIVE_INPUT") {
            if let Some(parsed) = parse_bool_value(&val) {
                self.general.predictive_input = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_AI_SESSION_RETENTION") {
            if let Some(parsed) = parse_positive_usize(&val) {
                self.general.ai_session_retention = parsed;
            }
        }

        if let Ok(val) = env::var("NTK_COLOR") {
            match val.to_lowercase().as_str() {
                "always" | "1" | "true" => self.display.color = ColorMode::Always,
                "never" | "0" | "false" => self.display.color = ColorMode::Never,
                _ => {} // keep auto
            }
        }

        // NO_COLOR spec: https://no-color.org/
        // Presence alone (any value including empty) means no color
        if env::var("NO_COLOR").is_ok() {
            self.display.color = ColorMode::Never;
        }

        if let Ok(val) = env::var("NTK_UNICODE") {
            match val.to_lowercase().as_str() {
                "always" | "1" | "true" => self.display.unicode = UnicodeMode::Always,
                "never" | "0" | "false" => self.display.unicode = UnicodeMode::Never,
                _ => {} // keep auto
            }
        }

        if let Ok(val) = env::var("NTK_TEMPLATE_DIR") {
            self.templates.directory = Some(val);
        }

        if let Ok(val) = env::var("NTK_SHELL") {
            self.shell.default_shell = Some(val);
        }
    }

    /// Get the default config file path
    ///
    /// - Linux/macOS: `~/.config/ntk/config.toml`
    /// - Windows: `%APPDATA%\ntk\config.toml`
    #[must_use]
    pub fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("ntk").join("config.toml"))
    }

    /// Get the default data directory for NTK
    ///
    /// - Linux/macOS: `~/.local/share/ntk`
    /// - Windows: `%APPDATA%\ntk\data`
    #[must_use]
    pub fn default_data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("ntk"))
    }

    /// Save current configuration to the default path
    ///
    /// # Errors
    ///
    /// Returns `Err` if the directory cannot be created or the file cannot be written.
    pub fn save(&self) -> crate::Result<PathBuf> {
        let path = Self::default_config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        self.save_to(&path)?;
        Ok(path)
    }

    /// Save current configuration to a specific path
    ///
    /// # Errors
    ///
    /// Returns `Err` if the directory cannot be created or the file cannot be written.
    pub fn save_to(&self, path: &Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;

        info!(path = %path.display(), "Configuration saved");
        Ok(())
    }

    /// Generate a default config file content for reference
    #[must_use]
    pub fn default_toml() -> String {
        toml::to_string_pretty(&Self::default()).unwrap_or_default()
    }

    /// Check if colors should be enabled based on config + terminal detection
    #[must_use]
    pub fn colors_enabled(&self) -> bool {
        match self.display.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => detect_color_support(),
        }
    }

    /// Check if Unicode output should be used based on config + terminal detection
    #[must_use]
    pub fn unicode_enabled(&self) -> bool {
        match self.display.unicode {
            UnicodeMode::Always => true,
            UnicodeMode::Never => false,
            UnicodeMode::Auto => detect_unicode_support(),
        }
    }

    /// Get the resolved template directory path
    #[must_use]
    pub fn template_dir(&self) -> Option<PathBuf> {
        self.templates
            .directory
            .as_ref()
            .map(|d| expand_tilde(d))
            .or_else(|| Self::default_data_dir().map(|d| d.join("templates")))
    }
}

// --- Terminal Detection ---

/// Detect whether the terminal supports color output.
///
/// Checks (in order):
/// 1. `NO_COLOR` env var (if set → no color) — <https://no-color.org/>
/// 2. `FORCE_COLOR` env var (if set → color)
/// 3. `TERM` env var (if "dumb" → no color)
/// 4. `std::io::IsTerminal` (if not a TTY → no color)
fn detect_color_support() -> bool {
    // NO_COLOR spec: any value (including empty) disables color
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }

    // FORCE_COLOR overrides other detection
    if env::var_os("FORCE_COLOR").is_some() {
        return true;
    }

    // TERM=dumb means basic terminal without color
    if let Ok(term) = env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stdout is an interactive terminal
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

/// Detect whether the terminal supports Unicode output.
///
/// On Windows, checks if running in Windows Terminal or modern `ConPTY`.
/// On Unix, checks locale settings for UTF-8.
fn detect_unicode_support() -> bool {
    // Windows: check for Windows Terminal or modern conhost
    #[cfg(windows)]
    {
        // WT_SESSION is set by Windows Terminal
        if env::var_os("WT_SESSION").is_some() {
            return true;
        }
        // ConEmuPID for ConEmu/Cmder
        if env::var_os("ConEmuPID").is_some() {
            return true;
        }
        // Default to ASCII on legacy Windows console
        false
    }

    #[cfg(not(windows))]
    {
        // Check LANG/LC_ALL for UTF-8
        if let Ok(lang) = env::var("LANG") {
            if lang.to_uppercase().contains("UTF") {
                return true;
            }
        }
        if let Ok(lc) = env::var("LC_ALL") {
            if lc.to_uppercase().contains("UTF") {
                return true;
            }
        }
        // Most modern Unix terminals support Unicode
        true
    }
}

/// Expand `~` prefix in path strings to the user's home directory
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Check if a string value is truthy
fn parse_bool_value(val: &str) -> Option<bool> {
    match val.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_positive_usize(val: &str) -> Option<usize> {
    let parsed = val.trim().parse::<usize>().ok()?;
    (parsed > 0).then_some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert!(!config.general.verbose);
        assert_eq!(config.general.log_level, "info");
        assert!(config.general.footer_output);
        assert_eq!(config.general.runtime_mode, RuntimeMode::Cli);
        assert!(!config.general.attention_bell);
        assert!(!config.general.attention_desktop_notification);
        assert!(!config.general.attention_unfocused_only);
        assert!(config.general.predictive_input);
        assert_eq!(config.general.ai_session_retention, 20);
        assert_eq!(config.display.color, ColorMode::Auto);
        assert_eq!(config.display.unicode, UnicodeMode::Auto);
        assert!(config.templates.directory.is_none());
        assert!(config.shell.default_shell.is_none());
    }

    #[test]
    fn config_roundtrip_toml() {
        let config = AppConfig {
            general: GeneralConfig {
                verbose: true,
                log_level: "debug".to_string(),
                footer_output: false,
                runtime_mode: RuntimeMode::Service,
                attention_bell: true,
                attention_desktop_notification: true,
                attention_unfocused_only: true,
                predictive_input: false,
                ai_session_retention: 5,
            },
            display: DisplayConfig {
                color: ColorMode::Always,
                unicode: UnicodeMode::Never,
            },
            templates: TemplateConfig {
                directory: Some("/custom/templates".to_string()),
            },
            shell: ShellConfig {
                default_shell: Some("zsh".to_string()),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn config_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let content = r#"
[general]
verbose = true
log_level = "debug"
footer_output = false
runtime_mode = "service"
attention_bell = true
attention_desktop_notification = true
attention_unfocused_only = true
predictive_input = false
ai_session_retention = 7

[display]
color = "never"
unicode = "always"

[templates]
directory = "/my/templates"

[shell]
default_shell = "fish"
"#;
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let config = AppConfig::load_from(&path).unwrap();
        assert!(config.general.verbose);
        assert_eq!(config.general.log_level, "debug");
        assert!(!config.general.footer_output);
        assert_eq!(config.general.runtime_mode, RuntimeMode::Service);
        assert!(config.general.attention_bell);
        assert!(config.general.attention_desktop_notification);
        assert!(config.general.attention_unfocused_only);
        assert!(!config.general.predictive_input);
        assert_eq!(config.general.ai_session_retention, 7);
        assert_eq!(config.display.color, ColorMode::Never);
        assert_eq!(config.display.unicode, UnicodeMode::Always);
        assert_eq!(
            config.templates.directory,
            Some("/my/templates".to_string())
        );
        assert_eq!(config.shell.default_shell, Some("fish".to_string()));
    }

    #[test]
    fn config_save_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("config.toml");

        let config = AppConfig {
            general: GeneralConfig {
                verbose: true,
                log_level: "warn".to_string(),
                footer_output: false,
                runtime_mode: RuntimeMode::Service,
                attention_bell: true,
                attention_desktop_notification: true,
                attention_unfocused_only: true,
                predictive_input: false,
                ai_session_retention: 9,
            },
            ..AppConfig::default()
        };

        config.save_to(&path).unwrap();
        assert!(path.exists());

        let loaded = AppConfig::load_from(&path).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn partial_config_uses_defaults() {
        let content = r"
[general]
verbose = true
";
        let config: AppConfig = toml::from_str(content).unwrap();
        assert!(config.general.verbose);
        assert_eq!(config.general.log_level, "info"); // default
        assert!(config.general.footer_output); // default
        assert_eq!(config.general.runtime_mode, RuntimeMode::Cli); // default
        assert!(!config.general.attention_bell); // default
        assert!(!config.general.attention_desktop_notification); // default
        assert!(!config.general.attention_unfocused_only); // default
        assert!(config.general.predictive_input); // default
        assert_eq!(config.general.ai_session_retention, 20); // default
        assert_eq!(config.display.color, ColorMode::Auto); // default
    }

    #[test]
    fn color_mode_display() {
        assert_eq!(format!("{}", ColorMode::Auto), "auto");
        assert_eq!(format!("{}", ColorMode::Always), "always");
        assert_eq!(format!("{}", ColorMode::Never), "never");
    }

    #[test]
    fn unicode_mode_display() {
        assert_eq!(format!("{}", UnicodeMode::Auto), "auto");
        assert_eq!(format!("{}", UnicodeMode::Always), "always");
        assert_eq!(format!("{}", UnicodeMode::Never), "never");
    }

    #[test]
    fn default_toml_is_parseable() {
        let toml_str = AppConfig::default_toml();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, AppConfig::default());
    }

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, PathBuf::from("/absolute/path"));

        // Tilde expansion depends on home dir being available
        let expanded = expand_tilde("~/templates");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join("templates"));
        }
    }

    #[test]
    fn parse_bool_value_handles_supported_values() {
        assert_eq!(parse_bool_value("1"), Some(true));
        assert_eq!(parse_bool_value("true"), Some(true));
        assert_eq!(parse_bool_value("True"), Some(true));
        assert_eq!(parse_bool_value("YES"), Some(true));
        assert_eq!(parse_bool_value("on"), Some(true));
        assert_eq!(parse_bool_value("  1  "), Some(true));
        assert_eq!(parse_bool_value("0"), Some(false));
        assert_eq!(parse_bool_value("false"), Some(false));
        assert_eq!(parse_bool_value("no"), Some(false));
        assert_eq!(parse_bool_value(""), None);
        assert_eq!(parse_bool_value("maybe"), None);
    }

    #[test]
    fn parse_positive_usize_accepts_only_nonzero_values() {
        assert_eq!(parse_positive_usize("1"), Some(1));
        assert_eq!(parse_positive_usize("20"), Some(20));
        assert_eq!(parse_positive_usize("0"), None);
        assert_eq!(parse_positive_usize("-1"), None);
        assert_eq!(parse_positive_usize("invalid"), None);
    }

    #[test]
    fn colors_enabled_respects_mode() {
        let mut config = AppConfig::default();

        config.display.color = ColorMode::Always;
        assert!(config.colors_enabled());

        config.display.color = ColorMode::Never;
        assert!(!config.colors_enabled());
    }

    #[test]
    fn unicode_enabled_respects_mode() {
        let mut config = AppConfig::default();

        config.display.unicode = UnicodeMode::Always;
        assert!(config.unicode_enabled());

        config.display.unicode = UnicodeMode::Never;
        assert!(!config.unicode_enabled());
    }
}
