//! Correlation id utilities.
//!
//! The generated identifiers are lightweight, process-local, and suitable for
//! correlating logs between high-level session and command boundaries.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static CORRELATION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a correlation id with the provided `prefix`.
///
/// The returned format is:
///
/// `"<prefix>-<unix-ms-hex>-<counter-hex>"`
///
/// Invalid/non-alphanumeric prefix characters are normalized to `-`.
/// If the normalized prefix is empty, `corr` is used.
#[must_use]
pub fn next_correlation_id(prefix: &str) -> String {
    let normalized_prefix = normalize_prefix(prefix);
    let timestamp_ms = unix_timestamp_millis();
    let sequence = CORRELATION_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{normalized_prefix}-{timestamp_ms:x}-{sequence:08x}")
}

fn normalize_prefix(prefix: &str) -> String {
    let sanitized = prefix
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();

    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "corr".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn next_correlation_id_uses_normalized_prefix() {
        let id = next_correlation_id("Session Main");
        assert!(id.starts_with("session-main-"));
    }

    #[test]
    fn next_correlation_id_uses_default_prefix_when_empty() {
        let id = next_correlation_id("   ");
        assert!(id.starts_with("corr-"));
    }

    #[test]
    fn next_correlation_id_is_unique_across_calls() {
        let mut seen = HashSet::new();
        for _ in 0..128 {
            let id = next_correlation_id("cmd");
            assert!(seen.insert(id), "duplicate correlation id generated");
        }
    }
}
