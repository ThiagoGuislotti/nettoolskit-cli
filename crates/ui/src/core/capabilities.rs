//! Terminal capability detection
//!
//! Detects the current terminal's color and Unicode rendering capabilities,
//! providing a cached global result used by all rendering code to decide
//! whether to emit ANSI escapes, owo-colors attributes, or plain text.
//!
//! # Detection Order
//!
//! 1. **Explicit override** — `NO_COLOR`, `FORCE_COLOR`, `NTK_COLOR`, `NTK_UNICODE`
//! 2. **Terminal probe** — `supports-color` crate + platform heuristics
//! 3. **Fallback** — safe ASCII-only, no-color defaults

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU8, Ordering};

// ---------------------------------------------------------------------------
// Color support level
// ---------------------------------------------------------------------------

/// Granularity of color support detected in the current terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ColorLevel {
    /// No color at all (piped output, NO_COLOR, dumb terminal)
    None = 0,
    /// Basic 16 ANSI colors (SGR 30-37, 90-97)
    Ansi16 = 1,
    /// Extended 256-color palette (SGR 38;5;n)
    Ansi256 = 2,
    /// Full 24-bit true-color (SGR 38;2;r;g;b)
    TrueColor = 3,
}

impl ColorLevel {
    /// Whether any color output is supported
    #[must_use]
    pub fn has_color(self) -> bool {
        self != Self::None
    }

    /// Whether 256-color or higher is available
    #[must_use]
    pub fn has_256(self) -> bool {
        self as u8 >= Self::Ansi256 as u8
    }

    /// Whether full true-color (24-bit) is available
    #[must_use]
    pub fn has_truecolor(self) -> bool {
        self == Self::TrueColor
    }
}

// ---------------------------------------------------------------------------
// Terminal capabilities (cached singleton)
// ---------------------------------------------------------------------------

/// Combined terminal capabilities
#[derive(Debug, Clone, Copy)]
pub struct TerminalCaps {
    /// Detected color support level
    pub color: ColorLevel,
    /// Whether the terminal can render Unicode (box-drawing, block chars)
    pub unicode: bool,
    /// Whether stdout is connected to an interactive terminal
    pub interactive: bool,
}

impl TerminalCaps {
    /// Probe the current terminal and return its capabilities
    #[must_use]
    fn detect() -> Self {
        let interactive = std::io::IsTerminal::is_terminal(&std::io::stdout());
        let color = detect_color_level(interactive);
        let unicode = detect_unicode();

        Self {
            color,
            unicode,
            interactive,
        }
    }
}

// ---------------------------------------------------------------------------
// Global cached access
// ---------------------------------------------------------------------------

/// Lazily detected terminal capabilities (computed once on first access)
static DETECTED: Lazy<TerminalCaps> = Lazy::new(TerminalCaps::detect);

// Runtime overrides (0 = not overridden, 1-4 = ColorLevel + 1)
static COLOR_OVERRIDE: AtomicU8 = AtomicU8::new(0);
// 0 = not overridden, 1 = force false, 2 = force true
static UNICODE_OVERRIDE: AtomicU8 = AtomicU8::new(0);

/// Get the current terminal capabilities
///
/// Returns the auto-detected values merged with any runtime overrides
/// set via [`set_color_override`] or [`set_unicode_override`].
#[must_use]
pub fn capabilities() -> TerminalCaps {
    let detected = *DETECTED;

    let color = match COLOR_OVERRIDE.load(Ordering::Relaxed) {
        0 => detected.color,
        v => match v.saturating_sub(1) {
            0 => ColorLevel::None,
            1 => ColorLevel::Ansi16,
            2 => ColorLevel::Ansi256,
            _ => ColorLevel::TrueColor,
        },
    };

    let unicode = match UNICODE_OVERRIDE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => detected.unicode,
    };

    TerminalCaps {
        color,
        unicode,
        interactive: detected.interactive,
    }
}

/// Override the detected color level at runtime
///
/// Typically called once during startup after loading user config.
/// Pass `None` to reset to auto-detected value.
pub fn set_color_override(level: Option<ColorLevel>) {
    let val = match level {
        None => 0,
        Some(l) => (l as u8) + 1,
    };
    COLOR_OVERRIDE.store(val, Ordering::Relaxed);
}

/// Override the detected Unicode support at runtime
///
/// Pass `None` to reset to auto-detected value.
pub fn set_unicode_override(enabled: Option<bool>) {
    let val = match enabled {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    };
    UNICODE_OVERRIDE.store(val, Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// Color detection logic
// ---------------------------------------------------------------------------

/// Detect color support level using `supports-color` crate + env overrides
fn detect_color_level(is_tty: bool) -> ColorLevel {
    // NO_COLOR spec: any value disables color — https://no-color.org/
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorLevel::None;
    }

    // TERM=dumb → no color
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return ColorLevel::None;
        }
    }

    // Non-interactive → no color (unless FORCE_COLOR is set)
    if !is_tty && std::env::var_os("FORCE_COLOR").is_none() {
        return ColorLevel::None;
    }

    // Use the `supports-color` crate for accurate probing
    match supports_color::on(supports_color::Stream::Stdout) {
        Some(level) => {
            if level.has_16m {
                ColorLevel::TrueColor
            } else if level.has_256 {
                ColorLevel::Ansi256
            } else {
                ColorLevel::Ansi16
            }
        }
        None => {
            // Fallback: if we have a TTY, assume basic ANSI support
            if is_tty {
                ColorLevel::Ansi16
            } else {
                ColorLevel::None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unicode detection logic
// ---------------------------------------------------------------------------

/// Detect if the terminal can render Unicode glyphs
fn detect_unicode() -> bool {
    #[cfg(windows)]
    {
        // Windows Terminal sets WT_SESSION
        if std::env::var_os("WT_SESSION").is_some() {
            return true;
        }
        // ConEmu/Cmder
        if std::env::var_os("ConEmuPID").is_some() {
            return true;
        }
        // VS Code integrated terminal
        if std::env::var("TERM_PROGRAM")
            .map(|v| v.contains("vscode"))
            .unwrap_or(false)
        {
            return true;
        }
        // Legacy cmd.exe / PowerShell console → ASCII only
        false
    }

    #[cfg(not(windows))]
    {
        // Check locale for UTF-8 support
        for var in &["LC_ALL", "LC_CTYPE", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let upper = val.to_uppercase();
                if upper.contains("UTF-8") || upper.contains("UTF8") {
                    return true;
                }
            }
        }
        // Most modern Unix terminals default to UTF-8
        true
    }
}

// ---------------------------------------------------------------------------
// Convenience helpers for conditional rendering
// ---------------------------------------------------------------------------

/// Returns `unicode` variant when Unicode is supported, `ascii` otherwise
#[must_use]
pub fn pick_char(unicode: char, ascii: char) -> char {
    if capabilities().unicode {
        unicode
    } else {
        ascii
    }
}

/// Returns `unicode` variant when Unicode is supported, `ascii` otherwise
#[must_use]
pub fn pick_str<'a>(unicode: &'a str, ascii: &'a str) -> &'a str {
    if capabilities().unicode {
        unicode
    } else {
        ascii
    }
}

/// Conditionally wrap text with ANSI color escape codes.
///
/// When color is disabled, returns the text unchanged.
#[must_use]
pub fn maybe_gray(text: &str) -> String {
    if capabilities().color.has_color() {
        format!("\x1b[90m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Guards tests that mutate global `COLOR_OVERRIDE` / `UNICODE_OVERRIDE`
    /// to prevent data-races when test threads run in parallel.
    static GLOBAL_STATE: Mutex<()> = Mutex::new(());

    #[test]
    fn color_level_ordering() {
        assert!(ColorLevel::None < ColorLevel::Ansi16);
        assert!(ColorLevel::Ansi16 < ColorLevel::Ansi256);
        assert!(ColorLevel::Ansi256 < ColorLevel::TrueColor);
    }

    #[test]
    fn color_level_queries() {
        assert!(!ColorLevel::None.has_color());
        assert!(ColorLevel::Ansi16.has_color());
        assert!(!ColorLevel::Ansi16.has_256());
        assert!(ColorLevel::Ansi256.has_256());
        assert!(ColorLevel::TrueColor.has_truecolor());
        assert!(!ColorLevel::Ansi256.has_truecolor());
    }

    #[test]
    fn override_color() {
        let _guard = GLOBAL_STATE.lock().unwrap();
        let orig = COLOR_OVERRIDE.load(Ordering::Relaxed);

        set_color_override(Some(ColorLevel::None));
        assert_eq!(capabilities().color, ColorLevel::None);

        set_color_override(Some(ColorLevel::TrueColor));
        assert_eq!(capabilities().color, ColorLevel::TrueColor);

        set_color_override(None);
        // Resets to auto-detected
        COLOR_OVERRIDE.store(orig, Ordering::Relaxed);
    }

    #[test]
    fn override_unicode() {
        let _guard = GLOBAL_STATE.lock().unwrap();
        let orig = UNICODE_OVERRIDE.load(Ordering::Relaxed);

        set_unicode_override(Some(false));
        assert!(!capabilities().unicode);

        set_unicode_override(Some(true));
        assert!(capabilities().unicode);

        set_unicode_override(None);
        UNICODE_OVERRIDE.store(orig, Ordering::Relaxed);
    }

    #[test]
    fn pick_char_delegates() {
        let _guard = GLOBAL_STATE.lock().unwrap();
        set_unicode_override(Some(true));
        assert_eq!(pick_char('╭', '+'), '╭');

        set_unicode_override(Some(false));
        assert_eq!(pick_char('╭', '+'), '+');

        set_unicode_override(None);
    }

    #[test]
    fn pick_str_delegates() {
        let _guard = GLOBAL_STATE.lock().unwrap();
        set_unicode_override(Some(true));
        assert_eq!(pick_str("───", "---"), "───");

        set_unicode_override(Some(false));
        assert_eq!(pick_str("───", "---"), "---");

        set_unicode_override(None);
    }

    #[test]
    fn maybe_gray_respects_color() {
        let _guard = GLOBAL_STATE.lock().unwrap();
        set_color_override(Some(ColorLevel::Ansi16));
        let result = maybe_gray("hello");
        assert!(result.contains("\x1b[90m"));

        set_color_override(Some(ColorLevel::None));
        let result = maybe_gray("hello");
        assert_eq!(result, "hello");

        set_color_override(None);
    }
}
