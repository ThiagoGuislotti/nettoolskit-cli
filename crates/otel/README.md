# nettoolskit-otel

> OpenTelemetry helpers and tracing integration for NetToolsKit CLI.

---

## Introduction

`nettoolskit-otel` provides observability utilities for the CLI, including tracing initialization and lightweight in-process metrics.
It is designed to be easy to integrate and safe to use in both interactive and non-interactive modes.

---

## Features

-   ✅ Tracing initialization helpers for common environments (`init_tracing`, `init_development_tracing`)
-   ✅ Metrics collector (`Metrics`) with counters, gauges, and timings
-   ✅ Timing helper (`Timer`) and macros (`time_operation!`, `log_operation!`)
-   ✅ Interactive-mode tracing support via `nettoolskit-ui::UiWriter`

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

---

## References

- https://docs.rs/tracing/
- https://docs.rs/tracing-subscriber/
- https://opentelemetry.io/
- https://github.com/ThiagoGuislotti/NetToolsKit/issues

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
