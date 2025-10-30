/// Telemetry utilities for NetToolsKit CLI
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, debug, trace};

/// Performance metrics collector with thread-safe operations
#[derive(Debug, Default)]
pub struct Metrics {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    timings: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl Clone for Metrics {
    fn clone(&self) -> Self {
        Self {
            counters: Arc::clone(&self.counters),
            gauges: Arc::clone(&self.gauges),
            timings: Arc::clone(&self.timings),
        }
    }
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        info!("Initializing metrics collector");
        Self::default()
    }

    /// Increment a counter with structured logging
    pub fn increment_counter(&self, name: impl Into<String>) {
        let name = name.into();
        let mut counters = self.counters.lock().unwrap();
        let new_value = *counters.entry(name.clone()).or_insert(0) + 1;
        counters.insert(name.clone(), new_value);

        debug!(
            counter = %name,
            value = new_value,
            "Counter incremented"
        );
    }

    /// Set a gauge value with structured logging
    pub fn set_gauge(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.clone(), value);

        debug!(
            gauge = %name,
            value = value,
            "Gauge value set"
        );
    }

    /// Record timing measurement
    pub fn record_timing(&self, name: impl Into<String>, duration: Duration) {
        let name = name.into();
        let mut timings = self.timings.lock().unwrap();
        timings.entry(name.clone()).or_insert_with(Vec::new).push(duration);

        debug!(
            timing = %name,
            duration_ms = duration.as_millis(),
            "Timing recorded"
        );
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.lock().unwrap();
        counters.get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(name).copied()
    }

    /// Get average timing for an operation
    pub fn get_average_timing(&self, name: &str) -> Option<Duration> {
        let timings = self.timings.lock().unwrap();
        if let Some(times) = timings.get(name) {
            if !times.is_empty() {
                let sum: Duration = times.iter().sum();
                return Some(sum / times.len() as u32);
            }
        }
        None
    }

    /// Get all counters snapshot
    pub fn counters_snapshot(&self) -> HashMap<String, u64> {
        let counters = self.counters.lock().unwrap();
        counters.clone()
    }

    /// Get all gauges snapshot
    pub fn gauges_snapshot(&self) -> HashMap<String, f64> {
        let gauges = self.gauges.lock().unwrap();
        gauges.clone()
    }

    /// Log metrics summary
    pub fn log_summary(&self) {
        let counters = self.counters_snapshot();
        let gauges = self.gauges_snapshot();

        info!(
            counter_count = counters.len(),
            gauge_count = gauges.len(),
            "Metrics summary logged"
        );

        for (name, value) in counters {
            trace!(counter = %name, value = value, "Counter value");
        }

        for (name, value) in gauges {
            trace!(gauge = %name, value = value, "Gauge value");
        }
    }
}

/// Timer utility for measuring operation duration
pub struct Timer {
    name: String,
    start: Instant,
    metrics: Metrics,
}

impl Timer {
    /// Start a new timer
    pub fn start(name: impl Into<String>, metrics: Metrics) -> Self {
        let name = name.into();
        debug!(operation = %name, "Starting timer");

        Self {
            name,
            start: Instant::now(),
            metrics,
        }
    }

    /// Stop the timer and record the measurement
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();

        info!(
            operation = %self.name,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );

        self.metrics.record_timing(&self.name, duration);
        duration
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        warn!(
            operation = %self.name,
            duration_ms = duration.as_millis(),
            "Timer dropped without explicit stop - auto-recording"
        );

        self.metrics.record_timing(&self.name, duration);
    }
}

/// Macro for easy structured logging with context
#[macro_export]
macro_rules! log_operation {
    ($level:ident, $operation:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            operation = $operation,
            $($key = $value,)*
        );
    };
}

/// Macro for timing operations with automatic cleanup
#[macro_export]
macro_rules! time_operation {
    ($metrics:expr, $name:expr, $block:expr) => {{
        let timer = $crate::telemetry::Timer::start($name, $metrics.clone());
        let result = $block;
        timer.stop();
        result
    }};
}