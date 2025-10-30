/// Telemetry utilities for NetToolsKit CLI
use std::collections::HashMap;

/// Performance metrics collector
#[derive(Debug, Default)]
pub struct Metrics {
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment a counter
    pub fn increment_counter(&mut self, name: impl Into<String>) {
        let name = name.into();
        *self.counters.entry(name).or_insert(0) += 1;
    }

    /// Set a gauge value
    pub fn set_gauge(&mut self, name: impl Into<String>, value: f64) {
        let name = name.into();
        self.gauges.insert(name, value);
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }

    /// Get all counters
    pub fn counters(&self) -> &HashMap<String, u64> {
        &self.counters
    }

    /// Get all gauges
    pub fn gauges(&self) -> &HashMap<String, f64> {
        &self.gauges
    }
}