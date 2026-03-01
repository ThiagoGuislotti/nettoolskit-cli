# nettoolskit-otel

> Telemetry, metrics, and structured logging for NetToolsKit CLI.

---

## Introduction

`nettoolskit-otel` provides observability utilities for the CLI, including structured logging via `tracing`, optional OTLP trace/metric export via OpenTelemetry, and a custom metrics/timer system.
It is designed to be easy to integrate and safe to use in both interactive and non-interactive modes.

> **Note:** Metrics remain purpose-built and in-process for CLI performance and simplicity.
> When configured, the same metrics are also mirrored to OTLP for centralized observability.

---

## Features

-   ✅ Tracing initialization helpers for common environments (`init_tracing`, `init_development_tracing`)
-   ✅ Optional OpenTelemetry OTLP trace and metric export (`OTEL_EXPORTER_OTLP_*` / `NTK_OTLP_*`)
-   ✅ Metrics collector (`Metrics`) with counters, gauges, and timings (custom, in-process)
-   ✅ Timing helper (`Timer`) with RAII auto-record and macros (`time_operation!`, `log_operation!`)
-   ✅ Correlation ID helper (`next_correlation_id`) for session/command log correlation
-   ✅ Interactive-mode tracing support via `nettoolskit-ui::UiWriter`

---

## Design Decision

This crate uses a hybrid approach:

| Aspect | Metrics (`Metrics`/`Timer`) | Traces (`tracing-opentelemetry`) |
|--------|------------------------------|----------------------------------|
| Runtime model | In-process API + optional OTLP mirror | Optional OTLP export |
| Network dependency | None unless OTLP metrics endpoint is set | None unless OTLP traces endpoint is set |
| Primary use | CLI counters/gauges/timings and runtime KPIs | Distributed trace correlation |
| Backends | Local memory and/or OTEL Collector-compatible backends | Jaeger, Grafana Tempo, OTEL Collector |

This keeps local CLI runs lightweight while enabling enterprise telemetry pipelines when needed.

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Initializing tracing](#example-1-initializing-tracing)
  - [Example 2: Recording metrics and timings](#example-2-recording-metrics-and-timings)
- [API Reference](#api-reference)
  - [Tracing Setup](#tracing-setup)
  - [Telemetry](#telemetry)
  - [Correlation IDs](#correlation-ids)
  - [Runtime Metrics Catalog](#runtime-metrics-catalog)
- [References](#references)
- [License](#license)

---

## Installation

### Via workspace path dependency

```toml
[dependencies]
nettoolskit-otel = { path = "../otel" }
```

### Via Git dependency

```toml
[dependencies]
nettoolskit-otel = { git = "https://github.com/ThiagoGuislotti/NetToolsKit", package = "nettoolskit-otel" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use anyhow::Result;
use nettoolskit_otel::init_tracing;

fn main() -> Result<()> {
	init_tracing(false)?;
	Ok(())
}
```

## OTLP Export Configuration

Trace export is enabled automatically when an OTLP traces endpoint is present:

- `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` (preferred)
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `NTK_OTLP_TRACES_ENDPOINT` (project-specific override)
- `NTK_OTLP_ENDPOINT` (project-specific fallback)

Optional protocol and timeout:

- `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` or `NTK_OTLP_TRACES_PROTOCOL`
- fallback: `OTEL_EXPORTER_OTLP_PROTOCOL` or `NTK_OTLP_PROTOCOL`
- values: `grpc` (default) or `http/protobuf`
- `OTEL_EXPORTER_OTLP_TRACES_TIMEOUT` or `NTK_OTLP_TRACES_TIMEOUT_MS`
- fallback: `OTEL_EXPORTER_OTLP_TIMEOUT` or `NTK_OTLP_TIMEOUT_MS`
- timeout unit: milliseconds (default: `10000`)

Metrics export is enabled automatically when a metrics endpoint is present:

- `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` (preferred)
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `NTK_OTLP_METRICS_ENDPOINT` (project-specific override)
- `NTK_OTLP_ENDPOINT` (project-specific fallback)

Optional protocol and timeout:

- `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` or `NTK_OTLP_METRICS_PROTOCOL`
- fallback: `OTEL_EXPORTER_OTLP_PROTOCOL` or `NTK_OTLP_PROTOCOL`
- values: `grpc` (default) or `http/protobuf`
- `OTEL_EXPORTER_OTLP_METRICS_TIMEOUT` or `NTK_OTLP_METRICS_TIMEOUT_MS`
- fallback: `OTEL_EXPORTER_OTLP_TIMEOUT` or `NTK_OTLP_TIMEOUT_MS`
- timeout unit: milliseconds (default: `10000`)

For short-lived executions, call `shutdown_tracing()` before process exit to flush pending OTLP data.

---

## Usage Examples

### Example 1: Initializing tracing

```rust
use anyhow::Result;
use nettoolskit_otel::{init_development_tracing, init_tracing_with_filter};

fn main() -> Result<()> {
	// Convenient presets:
	init_development_tracing()?;

	// Or fully customize with RUST_LOG-like filters:
	init_tracing_with_filter("nettoolskit=debug,info")?;
	Ok(())
}
```

### Example 2: Recording metrics and timings

```rust
use nettoolskit_otel::{Metrics, Timer, time_operation};

let metrics = Metrics::new();
metrics.increment_counter("commands_run");
metrics.set_gauge("queue_depth", 3.0);

let timer = Timer::start("render", metrics.clone());
// ... do work ...
let _duration = timer.stop();

time_operation!(metrics, "parse", {
	// ... do work ...
	42
});
```

---

## API Reference

### Tracing Setup

```rust
pub struct TracingConfig {
	pub verbose: bool,
	pub json_format: bool,
	pub with_file: bool,
	pub with_line_numbers: bool,
	pub service_name: String,
	pub service_version: String,
	pub interactive_mode: bool,
}

pub fn init_tracing(verbose: bool) -> anyhow::Result<()>;
pub fn init_tracing_with_config(config: TracingConfig) -> anyhow::Result<()>;
pub fn init_tracing_with_filter(filter: &str) -> anyhow::Result<()>;
pub fn init_production_tracing() -> anyhow::Result<()>;
pub fn init_development_tracing() -> anyhow::Result<()>;
pub fn shutdown_tracing();
```

### Telemetry

```rust
pub struct Metrics { /* fields omitted */ }
impl Metrics {
	pub fn new() -> Self;
	pub fn increment_counter(&self, name: impl Into<String>);
	pub fn set_gauge(&self, name: impl Into<String>, value: f64);
	pub fn record_timing(&self, name: impl Into<String>, duration: std::time::Duration);
	pub fn get_counter(&self, name: &str) -> u64;
	pub fn get_gauge(&self, name: &str) -> Option<f64>;
	pub fn get_average_timing(&self, name: &str) -> Option<std::time::Duration>;
	pub fn log_summary(&self);
}

pub struct Timer { /* fields omitted */ }
impl Timer {
	pub fn start(name: impl Into<String>, metrics: Metrics) -> Self;
	pub fn stop(self) -> std::time::Duration;
}
```

### Correlation IDs

```rust
pub fn next_correlation_id(prefix: &str) -> String;
```

### Runtime Metrics Catalog

The orchestrator emits a stable runtime/business metrics taxonomy:

- Counters:
  - `runtime_commands_total`
  - `runtime_commands_success_total`
  - `runtime_commands_error_total`
  - `runtime_commands_interrupted_total`
  - `runtime_command_<key>_total`
  - `runtime_command_<key>_{success|error|interrupted}_total`
- Timings:
  - `runtime_command_latency_all`
  - `runtime_command_latency_<key>`
- Gauges:
  - `runtime_command_success_rate_pct`
  - `runtime_command_error_rate_pct`
  - `runtime_command_cancellation_rate_pct`
  - `runtime_last_command_duration_ms`
  - `runtime_command_avg_latency_ms`
  - `runtime_command_<key>_avg_latency_ms`

---

## References

- https://docs.rs/tracing/
- https://docs.rs/tracing-subscriber/
- https://github.com/ThiagoGuislotti/NetToolsKit/issues

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
