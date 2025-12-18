# nettoolskit-core

> Core types and utilities shared across the NetToolsKit CLI workspace.

---

## Introduction

`nettoolskit-core` contains foundational types and small utilities used by the other crates in this workspace.
It is focused on shared abstractions (menu traits), standardized exit status, and feature detection.

---

## Features

-   ✅ Standard `ExitStatus` type shared across crates
-   ✅ Menu traits (`MenuEntry`, `MenuProvider`, `CommandEntry`) to keep UI decoupled from domain types
-   ✅ Runtime feature detection (`Features`) via compile-time flags and environment variables
-   ✅ Shared utility modules (async utils, file search, path utils, string utils)

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Detecting features](#example-1-detecting-features)
  - [Example 2: Implementing menu traits](#example-2-implementing-menu-traits)
- [API Reference](#api-reference)
  - [Main Types](#main-types)
  - [Menu Traits](#menu-traits)
- [References](#references)
- [License](#license)

---

## Installation

### Via workspace path dependency

```toml
[dependencies]
nettoolskit-core = { path = "../core" }
```

### Via Git dependency

```toml
[dependencies]
nettoolskit-core = { git = "https://github.com/ThiagoGuislotti/NetToolsKit", package = "nettoolskit-core" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_core::{ExitStatus, Features};

let features = Features::detect();
let status = ExitStatus::Success;

println!("features: {}", features.description());
println!("exit: {}", i32::from(status));
```

---

## Usage Examples

### Example 1: Detecting features

```rust
use nettoolskit_core::Features;

let features = Features::detect();

if features.has_any_modern() {
	println!("Modern features enabled: {}", features.description());
} else {
	println!("Using default UI");
}
```

### Example 2: Implementing menu traits

```rust
use nettoolskit_core::{MenuEntry, MenuProvider};

#[derive(Clone)]
struct Action {
	label: &'static str,
	description: &'static str,
}

impl std::fmt::Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.label)
	}
}

impl MenuEntry for Action {
	fn label(&self) -> &str {
		self.label
	}

	fn description(&self) -> &str {
		self.description
	}
}

impl MenuProvider for Action {
	fn menu_items() -> Vec<String> {
		Self::all_variants()
			.into_iter()
			.map(|a| format!("{} - {}", a.label(), a.description()))
			.collect()
	}

	fn all_variants() -> Vec<Self> {
		vec![
			Self { label: "/help", description: "Show help" },
			Self { label: "/quit", description: "Exit" },
		]
	}
}
```

---

## API Reference

### Main Types

```rust
pub type Result<T> = anyhow::Result<T>;

pub enum ExitStatus {
	Success,
	Error,
	Interrupted,
}

pub struct Features {
	pub use_modern_tui: bool,
	pub use_event_driven: bool,
	pub use_frame_scheduler: bool,
	pub use_persistent_sessions: bool,
}

impl Features {
	pub fn detect() -> Self;
	pub const fn is_full_modern(&self) -> bool;
	pub const fn has_any_modern(&self) -> bool;
	pub fn description(&self) -> String;
	pub fn print_status(&self);
}
```

### Menu Traits

```rust
pub trait MenuEntry {
	fn label(&self) -> &str;
	fn description(&self) -> &str;
}

pub trait MenuProvider: MenuEntry + Clone + std::fmt::Display {
	fn menu_items() -> Vec<String>
	where
		Self: Sized;

	fn all_variants() -> Vec<Self>
	where
		Self: Sized;
}

pub trait CommandEntry: MenuEntry + Into<&'static str> + Copy {
	fn name(&self) -> &'static str;
	fn slash_static(&self) -> String;
}
```

---

## References

- https://doc.rust-lang.org/book/
- https://docs.rs/anyhow/
- https://docs.rs/tokio/
- https://github.com/ThiagoGuislotti/NetToolsKit/issues

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
