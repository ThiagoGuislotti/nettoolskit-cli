use nettoolskit_otel::Metrics;

#[test]
fn test_metrics_creation() {
    let metrics = Metrics::new();
    assert!(metrics.counters().is_empty());
    assert!(metrics.gauges().is_empty());

    let default_metrics = Metrics::default();
    assert!(default_metrics.counters().is_empty());
    assert!(default_metrics.gauges().is_empty());
}

#[test]
fn test_counter_operations() {
    let mut metrics = Metrics::new();

    // Test initial state
    assert_eq!(metrics.get_counter("test_counter"), 0);

    // Test increment
    metrics.increment_counter("test_counter");
    assert_eq!(metrics.get_counter("test_counter"), 1);

    // Test multiple increments
    metrics.increment_counter("test_counter");
    metrics.increment_counter("test_counter");
    assert_eq!(metrics.get_counter("test_counter"), 3);

    // Test different counter
    metrics.increment_counter("another_counter");
    assert_eq!(metrics.get_counter("another_counter"), 1);
    assert_eq!(metrics.get_counter("test_counter"), 3);
}

#[test]
fn test_counter_with_string_types() {
    let mut metrics = Metrics::new();

    // Test with &str
    metrics.increment_counter("str_counter");
    assert_eq!(metrics.get_counter("str_counter"), 1);

    // Test with String
    metrics.increment_counter("string_counter".to_string());
    assert_eq!(metrics.get_counter("string_counter"), 1);

    // Test with formatted string
    let name = format!("formatted_{}", "counter");
    metrics.increment_counter(name);
    assert_eq!(metrics.get_counter("formatted_counter"), 1);
}

#[test]
fn test_gauge_operations() {
    let mut metrics = Metrics::new();

    // Test initial state
    assert_eq!(metrics.get_gauge("test_gauge"), None);

    // Test set gauge
    metrics.set_gauge("test_gauge", 3.14);
    assert_eq!(metrics.get_gauge("test_gauge"), Some(3.14));

    // Test overwrite gauge
    metrics.set_gauge("test_gauge", 2.71);
    assert_eq!(metrics.get_gauge("test_gauge"), Some(2.71));

    // Test different gauges
    metrics.set_gauge("another_gauge", 1.41);
    assert_eq!(metrics.get_gauge("another_gauge"), Some(1.41));
    assert_eq!(metrics.get_gauge("test_gauge"), Some(2.71));
}

#[test]
fn test_gauge_with_string_types() {
    let mut metrics = Metrics::new();

    // Test with &str
    metrics.set_gauge("str_gauge", 1.0);
    assert_eq!(metrics.get_gauge("str_gauge"), Some(1.0));

    // Test with String
    metrics.set_gauge("string_gauge".to_string(), 2.0);
    assert_eq!(metrics.get_gauge("string_gauge"), Some(2.0));

    // Test with formatted string
    let name = format!("formatted_{}", "gauge");
    metrics.set_gauge(name, 3.0);
    assert_eq!(metrics.get_gauge("formatted_gauge"), Some(3.0));
}

#[test]
fn test_mixed_operations() {
    let mut metrics = Metrics::new();

    // Mix counters and gauges
    metrics.increment_counter("requests");
    metrics.set_gauge("cpu_usage", 0.75);
    metrics.increment_counter("requests");
    metrics.set_gauge("memory_usage", 0.85);

    assert_eq!(metrics.get_counter("requests"), 2);
    assert_eq!(metrics.get_gauge("cpu_usage"), Some(0.75));
    assert_eq!(metrics.get_gauge("memory_usage"), Some(0.85));

    // Verify collections
    assert_eq!(metrics.counters().len(), 1);
    assert_eq!(metrics.gauges().len(), 2);
}

#[test]
fn test_collections_access() {
    let mut metrics = Metrics::new();

    metrics.increment_counter("counter1");
    metrics.increment_counter("counter2");
    metrics.increment_counter("counter1");

    metrics.set_gauge("gauge1", 1.0);
    metrics.set_gauge("gauge2", 2.0);

    let counters = metrics.counters();
    assert_eq!(counters.len(), 2);
    assert_eq!(counters.get("counter1"), Some(&2));
    assert_eq!(counters.get("counter2"), Some(&1));

    let gauges = metrics.gauges();
    assert_eq!(gauges.len(), 2);
    assert_eq!(gauges.get("gauge1"), Some(&1.0));
    assert_eq!(gauges.get("gauge2"), Some(&2.0));
}

#[test]
fn test_nonexistent_keys() {
    let metrics = Metrics::new();

    assert_eq!(metrics.get_counter("nonexistent"), 0);
    assert_eq!(metrics.get_gauge("nonexistent"), None);
}

#[test]
fn test_debug_format() {
    let mut metrics = Metrics::new();
    metrics.increment_counter("test");
    metrics.set_gauge("test_gauge", 1.5);

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("Metrics"));
    assert!(!debug_str.is_empty());
}

#[test]
fn test_large_numbers() {
    let mut metrics = Metrics::new();

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