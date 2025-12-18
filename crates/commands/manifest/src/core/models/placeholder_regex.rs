//! Regex for placeholder detection in templates

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for placeholder detection in templates
#[allow(dead_code)]
pub static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{([A-Za-z0-9_]+)\}").expect("invalid placeholder regex"));
