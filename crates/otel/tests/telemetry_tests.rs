//! Tests for Metrics telemetry functionality
//!
//! Validates counter operations (increment, get), gauge operations (set, get),
//! string type compatibility (&str, String, formatted), collection snapshots
//! (counters_snapshot, gauges_snapshot), and edge cases (nonexistent keys,
//! large numbers, debug formatting).
//!
//! ## Test Coverage
//! - Metrics creation (new, default)
//! - Counter operations (increment, get, multiple keys)
//! - Gauge operations (set, get, overwrite)
//! - String type handling (&str, String, formatted)
//! - Mixed operations (counters + gauges)
//! - Collection access (snapshots)
//! - Edge cases (nonexistent keys, large numbers)

use nettoolskit_otel::Metrics;

// Metrics Creation Tests

#[test]
fn test_metrics_creation() {
    // Arrange & Act
    let metrics = Metrics::new();
    let default_metrics = Metrics::default();

    // Assert
    assert!(metrics.counters_snapshot().is_empty());
    assert!(metrics.gauges_snapshot().is_empty());
    assert!(default_metrics.counters_snapshot().is_empty());
    assert!(default_metrics.gauges_snapshot().is_empty());
}

// Counter Operations Tests

#[test]
fn test_counter_operations() {
    // Arrange
    let metrics = Metrics::new();

    // Act & Assert - Initial state
    assert_eq!(metrics.get_counter("test_counter"), 0);

    // Act - First increment
    metrics.increment_counter("test_counter");

    // Assert
    assert_eq!(metrics.get_counter("test_counter"), 1);

    // Act - Multiple increments
    metrics.increment_counter("test_counter");
    metrics.increment_counter("test_counter");

    // Assert
    assert_eq!(metrics.get_counter("test_counter"), 3);

    // Act - Different counter
    metrics.increment_counter("another_counter");

    // Assert
    assert_eq!(metrics.get_counter("another_counter"), 1);
    assert_eq!(metrics.get_counter("test_counter"), 3);
}

#[test]
fn test_counter_with_string_types() {
    // Arrange
    let metrics = Metrics::new();

    // Act - &str
    metrics.increment_counter("str_counter");

    // Assert
    assert_eq!(metrics.get_counter("str_counter"), 1);

    // Act - String
    metrics.increment_counter("string_counter".to_string());

    // Assert
    assert_eq!(metrics.get_counter("string_counter"), 1);

    // Act - Formatted string
    let name = format!("formatted_{}", "counter");
    metrics.increment_counter(name);

    // Assert
    assert_eq!(metrics.get_counter("formatted_counter"), 1);
}

// Gauge Operations Tests

#[test]
fn test_gauge_operations() {
    // Arrange
    let metrics = Metrics::new();

    // Act & Assert - Initial state
    assert_eq!(metrics.get_gauge("test_gauge"), None);

    // Act - Set gauge
    metrics.set_gauge("test_gauge", 3.14);

    // Assert
    assert_eq!(metrics.get_gauge("test_gauge"), Some(3.14));

    // Act - Overwrite gauge
    metrics.set_gauge("test_gauge", 2.71);

    // Assert
    assert_eq!(metrics.get_gauge("test_gauge"), Some(2.71));

    // Act - Different gauges
    metrics.set_gauge("another_gauge", 1.41);

    // Assert
    assert_eq!(metrics.get_gauge("another_gauge"), Some(1.41));
    assert_eq!(metrics.get_gauge("test_gauge"), Some(2.71));
}

#[test]
fn test_gauge_with_string_types() {
    // Arrange
    let metrics = Metrics::new();

    // Act - &str
    metrics.set_gauge("str_gauge", 1.0);

    // Assert
    assert_eq!(metrics.get_gauge("str_gauge"), Some(1.0));

    // Act - String
    metrics.set_gauge("string_gauge".to_string(), 2.0);

    // Assert
    assert_eq!(metrics.get_gauge("string_gauge"), Some(2.0));

    // Act - Formatted string
    let name = format!("formatted_{}", "gauge");
    metrics.set_gauge(name, 3.0);

    // Assert
    assert_eq!(metrics.get_gauge("formatted_gauge"), Some(3.0));
}

// Mixed Operations Tests

#[test]
fn test_mixed_operations() {
    // Arrange
    let metrics = Metrics::new();

    // Act
    metrics.increment_counter("requests");
    metrics.set_gauge("cpu_usage", 0.75);
    metrics.increment_counter("requests");
    metrics.set_gauge("memory_usage", 0.85);

    // Assert
    assert_eq!(metrics.get_counter("requests"), 2);
    assert_eq!(metrics.get_gauge("cpu_usage"), Some(0.75));
    assert_eq!(metrics.get_gauge("memory_usage"), Some(0.85));
    assert_eq!(metrics.counters_snapshot().len(), 1);
    assert_eq!(metrics.gauges_snapshot().len(), 2);
}

#[test]
fn test_collections_access() {
    // Arrange
    let metrics = Metrics::new();

    // Act
    metrics.increment_counter("counter1");
    metrics.increment_counter("counter2");
    metrics.increment_counter("counter1");
    metrics.set_gauge("gauge1", 1.0);
    metrics.set_gauge("gauge2", 2.0);

    // Assert
    let counters = metrics.counters_snapshot();
    assert_eq!(counters.len(), 2);
    assert_eq!(counters.get("counter1"), Some(&2));
    assert_eq!(counters.get("counter2"), Some(&1));

    let gauges = metrics.gauges_snapshot();
    assert_eq!(gauges.len(), 2);
    assert_eq!(gauges.get("gauge1"), Some(&1.0));
    assert_eq!(gauges.get("gauge2"), Some(&2.0));
}

// Edge Cases Tests

#[test]
fn test_nonexistent_keys() {
    // Arrange
    let metrics = Metrics::new();

    // Assert
    assert_eq!(metrics.get_counter("nonexistent"), 0);
    assert_eq!(metrics.get_gauge("nonexistent"), None);
}

#[test]
fn test_debug_format() {
    // Arrange
    let metrics = Metrics::new();

    // Act
    metrics.increment_counter("test");
    metrics.set_gauge("test_gauge", 1.5);
    let debug_str = format!("{:?}", metrics);

    // Assert
    assert!(debug_str.contains("Metrics"));
    assert!(!debug_str.is_empty());
}

#[test]
fn test_large_numbers() {
    let metrics = Metrics::new();

    // Test large counter values
    for _ in 0..1000 {
        metrics.increment_counter("large_counter");
    }
    assert_eq!(metrics.get_counter("large_counter"), 1000);

    // Test extreme gauge values
    metrics.set_gauge("large_gauge", f64::MAX);
    assert_eq!(metrics.get_gauge("large_gauge"), Some(f64::MAX));

    metrics.set_gauge("small_gauge", f64::MIN);
    assert_eq!(metrics.get_gauge("small_gauge"), Some(f64::MIN));
}
