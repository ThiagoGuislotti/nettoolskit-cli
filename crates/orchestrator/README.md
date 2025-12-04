# Orchestrator

Command orchestration layer for NetToolsKit CLI.

## Purpose

This crate provides the orchestration layer between the CLI interface and command implementations:
- Command dispatch and routing
- Async command execution with progress tracking
- Command models and menu system

## Architecture

```
┌─────────────┐
│     CLI     │  (User Interface)
│  display    │
│   events    │
│   input     │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│ Orchestrator│  (THIS CRATE)
│   models    │  ← MainAction, Command, ExitStatus
│  execution  │  ← processor, executor
└─────┬───────┘
      │
      ▼
┌─────────────┐
│  Commands   │  (Business Logic)
│    help     │
│  manifest   │
│  translate  │
└─────────────┘
```

## Modules

### `models`
- `MainAction`: Main menu action enum
- `Command`: Type alias for backward compatibility
- `ExitStatus`: Command execution result
- `get_main_action()`, `get_command()`: Command parsing

### `execution`
- `processor`: Command routing and dispatch
- `executor`: Async command execution with progress tracking

## Usage

```rust
use nettoolskit_orchestrator::{process_command, MainAction, ExitStatus};

// Process a command
let status: ExitStatus = process_command("/help").await;

// Get command enum
if let Some(MainAction::Quit) = get_command("/quit") {
    // Handle quit
}
```

## Design Principles

1. **Separation of Concerns**: Orchestration logic separate from UI and business logic
2. **Async-First**: Non-blocking command execution
3. **Progress Tracking**: Built-in support for long-running operations
4. **Type Safety**: Strong typing for command models and states
