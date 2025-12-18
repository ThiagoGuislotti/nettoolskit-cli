# nettoolskit-ui

> Terminal UI components for NetToolsKit CLI.

---

## Introduction

`nettoolskit-ui` provides reusable terminal UI building blocks used by the CLI.
It focuses on consistent rendering (boxes/menus), interactive prompts, and terminal layout management.

---

## Features

-   ✅ Reusable rendering components (`render_box`, `render_interactive_menu`, `render_enum_menu`)
-   ✅ Command palette for menu-like command discovery (`CommandPalette`)
-   ✅ Prompt helpers for consistent input UX (`render_prompt`, `get_prompt_string`)
-   ✅ Terminal layout helpers for interactive mode (footer logging, scroll regions)

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Rendering a prompt](#example-1-rendering-a-prompt)
  - [Example 2: Building a command palette](#example-2-building-a-command-palette)
- [API Reference](#api-reference)
  - [Rendering](#rendering)
  - [Interaction](#interaction)
  - [Terminal](#terminal)
- [References](#references)
- [License](#license)

---

## Installation

### Via workspace path dependency

```toml
[dependencies]
nettoolskit-ui = { path = "../ui" }
```

### Via Git dependency

```toml
[dependencies]
nettoolskit-ui = { git = "https://github.com/ThiagoGuislotti/NetToolsKit", package = "nettoolskit-ui" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_ui::render_prompt;

render_prompt().expect("failed to render prompt");
println!("Type /help and press Enter");
```

---

## Usage Examples

### Example 1: Rendering a prompt

```rust
use nettoolskit_ui::{get_prompt_string, render_prompt_with_command};

println!("{}", get_prompt_string());
render_prompt_with_command("/help").expect("failed to render prompt");
```

### Example 2: Building a command palette

```rust
use nettoolskit_core::MenuEntry;
use nettoolskit_ui::CommandPalette;

#[derive(Clone)]
struct Item {
	label: &'static str,
	description: &'static str,
}

impl MenuEntry for Item {
	fn label(&self) -> &str {
		self.label
	}

	fn description(&self) -> &str {
		self.description
	}
}

let entries = vec![
	Item { label: "/help", description: "Show help" },
	Item { label: "/quit", description: "Exit" },
];

let palette = CommandPalette::new(entries)
	.with_title("Commands")
	.with_subtitle("Select an option")
	.with_prompt("Select →");

let selected = palette.show();
println!("selected: {:?}", selected);
```

---

## API Reference

### Rendering

```rust
pub struct BoxConfig { /* fields omitted */ }
impl BoxConfig {
	pub fn new(title: impl Into<String>) -> Self;
	pub fn with_title_color(self, color: owo_colors::Rgb) -> Self;
	pub fn with_subtitle(self, subtitle: impl Into<String>) -> Self;
	pub fn with_title_prefix(self, prefix: impl Into<String>) -> Self;
	pub fn add_footer_item(self, label: impl Into<String>, value: impl Into<String>, color: owo_colors::Rgb) -> Self;
	pub fn with_border_color(self, color: owo_colors::Rgb) -> Self;
	pub fn with_width(self, width: usize) -> Self;
	pub fn with_spacing(self, add_spacing: bool) -> Self;
}
pub fn render_box(config: BoxConfig);

pub struct MenuConfig<T> { /* fields omitted */ }
impl<T> MenuConfig<T> {
	pub fn new(prompt: impl Into<String>, items: Vec<T>) -> Self;
	pub fn with_cursor_color(self, color: owo_colors::Rgb) -> Self;
	pub fn with_help_message(self, message: impl Into<String>) -> Self;
	pub fn with_page_size(self, size: usize) -> Self;
}
pub fn render_interactive_menu<T>(config: MenuConfig<T>) -> Result<T, inquire::InquireError>
where
	T: std::fmt::Display + Clone;

pub struct EnumMenuConfig { /* fields omitted */ }
impl EnumMenuConfig {
	pub fn new(title: impl Into<String>, subtitle: impl Into<String>, current_dir: impl Into<String>) -> Self;
	pub fn with_theme_color(self, color: owo_colors::Rgb) -> Self;
	pub fn with_width(self, width: usize) -> Self;
	pub fn add_footer_item(self, key: impl Into<String>, value: impl Into<String>, color: owo_colors::Rgb) -> Self;
}
pub fn render_enum_menu<T>(config: EnumMenuConfig) -> Result<T, inquire::InquireError>
where
	T: nettoolskit_core::MenuProvider + std::fmt::Display;
```

### Interaction

```rust
pub struct CommandPalette { /* fields omitted */ }
impl CommandPalette {
	pub fn new<T: nettoolskit_core::MenuEntry>(entries: Vec<T>) -> Self;
	pub fn with_title(self, title: impl Into<String>) -> Self;
	pub fn with_subtitle(self, subtitle: impl Into<String>) -> Self;
	pub fn with_directory(self, directory: impl Into<String>) -> Self;
	pub fn with_prompt(self, prompt: impl Into<String>) -> Self;
	pub fn reload_entries<T: nettoolskit_core::MenuEntry>(&mut self, entries: Vec<T>);
	pub fn show(&self) -> Option<String>;
}

pub fn render_prompt() -> std::io::Result<()>;
pub fn render_prompt_with_command(cmd: &str) -> std::io::Result<()>;
pub fn get_prompt_string() -> String;
pub fn get_prompt_symbol() -> &'static str;

pub struct UiWriter { /* fields omitted */ }
impl UiWriter {
	pub fn new() -> Self;
}
```

### Terminal

```rust
pub fn clear_terminal() -> std::io::Result<()>;

pub struct InteractiveLogGuard { /* fields omitted */ }
impl InteractiveLogGuard {
	pub fn deactivate(&mut self);
}
pub fn begin_interactive_logging() -> InteractiveLogGuard;
pub fn disable_interactive_logging();

pub struct TerminalLayout { /* fields omitted */ }
impl TerminalLayout {
	pub fn initialize<F>(render_header: Option<F>) -> std::io::Result<Self>
	where
		F: FnOnce();

	pub fn append_log_line(line: &str) -> std::io::Result<()>;
}
```

---

## References

- https://docs.rs/crossterm/
- https://docs.rs/inquire/
- https://github.com/ThiagoGuislotti/NetToolsKit/issues

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
