# nettoolskit-orchestrator

> Command orchestration layer for NetToolsKit CLI.

---

## Introduction

`nettoolskit-orchestrator` is the “glue” between the CLI UX and the command implementations.
It provides models for command routing and an async execution layer that can run work concurrently and report progress.

---

## Features

-   ✅ Command parsing and dispatch (`MainAction`, `get_main_action`, `process_command`)
-   ✅ Async execution utilities (`AsyncCommandExecutor`, `CommandHandle`)
-   ✅ Progress reporting primitives (`CommandProgress`, `ProgressSender`)
-   ✅ Backward-compatible exports (`Command`, `get_command`)

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Parsing actions](#example-1-parsing-actions)
  - [Example 2: Running async work with progress](#example-2-running-async-work-with-progress)
- [API Reference](#api-reference)
  - [Models](#models)
  - [Processor](#processor)
  - [Executor](#executor)
- [References](#references)
- [License](#license)

---

## Installation

### Via workspace path dependency

```toml
[dependencies]
nettoolskit-orchestrator = { path = "../orchestrator" }
```

### Via Git dependency

```toml
[dependencies]
nettoolskit-orchestrator = { git = "https://github.com/ThiagoGuislotti/NetToolsKit", package = "nettoolskit-orchestrator" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_orchestrator::get_main_action;

let action = get_main_action("/help");
println!("action: {:?}", action);
```

---

## Usage Examples

### Example 1: Parsing actions

```rust
use nettoolskit_orchestrator::{get_main_action, MainAction};

assert_eq!(get_main_action("/help"), Some(MainAction::Help));
assert_eq!(get_main_action("/manifest list"), Some(MainAction::Manifest));
assert_eq!(get_main_action("/quit"), Some(MainAction::Quit));
```

### Example 2: Running async work with progress

```rust,no_run
use nettoolskit_orchestrator::{AsyncCommandExecutor, CommandProgress};

#[tokio::main]
async fn main() {
      let mut executor = AsyncCommandExecutor::new();

      let (handle, mut progress) = executor.spawn_with_progress(|tx| async move {
            let _ = tx.send(CommandProgress::message("Starting..."));
            let _ = tx.send(CommandProgress::percent("Downloading", 50));
            Ok("done".to_string())
      });

      while let Some(update) = progress.recv().await {
            println!("progress: {}", update.message);
      }

      let _ = handle.wait().await;
}
```

---

## API Reference

### Models

```rust
pub enum MainAction {
      Help,
      Manifest,
      Translate,
      Quit,
}

impl MainAction {
      pub fn description(&self) -> &'static str;
}

pub fn get_main_action(slash: &str) -> Option<MainAction>;

pub use main_action::MainAction as Command;
pub use main_action::get_main_action as get_command;
pub use nettoolskit_core::ExitStatus;
```

### Processor

```rust
pub async fn process_command(cmd: &str) -> nettoolskit_core::ExitStatus;
pub async fn process_text(text: &str) -> nettoolskit_core::ExitStatus;
```

### Executor

```rust
pub type CommandResult = Result<String, Box<dyn std::error::Error + Send + Sync>>;

pub struct CommandHandle { /* fields omitted */ }
impl CommandHandle {
      pub fn new(receiver: tokio::sync::oneshot::Receiver<CommandResult>) -> Self;
      pub fn cancellable(receiver: tokio::sync::oneshot::Receiver<CommandResult>, cancel_tx: tokio::sync::mpsc::Sender<()>) -> Self;
      pub async fn wait(self) -> Result<CommandResult, tokio::sync::oneshot::error::RecvError>;
      pub fn try_result(&mut self) -> Option<CommandResult>;
      pub async fn cancel(&mut self) -> bool;
}

pub struct CommandProgress {
      pub message: String,
      pub percent: Option<u8>,
      pub total: Option<usize>,
      pub completed: Option<usize>,
}
impl CommandProgress {
      pub fn message(msg: impl Into<String>) -> Self;
      pub fn percent(msg: impl Into<String>, percent: u8) -> Self;
      pub fn steps(msg: impl Into<String>, completed: usize, total: usize) -> Self;
}

pub type ProgressSender = tokio::sync::mpsc::UnboundedSender<CommandProgress>;

pub struct AsyncCommandExecutor { /* fields omitted */ }
impl AsyncCommandExecutor {
      pub fn new() -> Self;
      pub fn with_limit(max_concurrent: usize) -> Self;
      pub fn spawn<F>(&mut self, future: F) -> CommandHandle
      where
            F: std::future::Future<Output = CommandResult> + Send + 'static;
      pub fn spawn_cancellable<F>(&mut self, future: F) -> CommandHandle
      where
            F: std::future::Future<Output = CommandResult> + Send + 'static;
      pub fn spawn_with_progress<F, Fut>(&mut self, factory: F) -> (CommandHandle, tokio::sync::mpsc::UnboundedReceiver<CommandProgress>)
      where
            F: FnOnce(ProgressSender) -> Fut + Send + 'static,
            Fut: std::future::Future<Output = CommandResult> + Send + 'static;
      pub fn is_full(&self) -> bool;
      pub fn running_count(&self) -> usize;
      pub async fn wait_all(&mut self);
      pub async fn cancel_all(&mut self);
}
```

---

## References

- https://docs.rs/tokio/
- https://docs.rs/strum/
- https://github.com/ThiagoGuislotti/NetToolsKit/issues

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
