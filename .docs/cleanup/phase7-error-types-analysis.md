# Phase 7: Error Type Consolidation - Analysis

**Date:** 2025-11-03
**Status:** 🔍 Analysis
**Codex Alignment:** ✅ Validated

---

## Executive Summary

Analysis of error handling patterns across NetToolsKit workspace to adopt **thiserror** for library errors and **anyhow** for application code, following Codex patterns.

**Current State:**
- ✅ `anyhow` already used in 4 crates (core, otel, ollama, file-search)
- ⚠️ `commands` crate uses `Box<dyn Error>` (should use thiserror)
- ✅ Error patterns partially align with Codex (60% match)

**Goal:** 100% alignment with Codex error handling patterns

---

## Codex Patterns Analysis

### Pattern 1: thiserror for Library Errors

**Evidence from Codex:**
```rust
// codex-rs/core/src/error.rs
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CodexErr>;

#[derive(Error, Debug)]
pub enum CodexErr {
    #[error("turn aborted")]
    TurnAborted { dangling_artifacts: Vec<ProcessedResponseItem> },

    #[error("stream disconnected: {0}")]
    Stream(String, Option<Duration>),

    #[error("context window exceeded")]
    ContextWindowExceeded,
    // ... 30+ variants
}
```

**Pattern:**
- Dedicated `error.rs` file in library crates
- `thiserror::Error` derive macro
- Descriptive error messages with `#[error()]`
- Type-safe variants with structured data
- Custom `Result<T>` type alias

**Usage:** `codex-rs/core/`, `codex-rs/git-tooling/`, `codex-rs/apply-patch/`, `codex-rs/utils/image/`

---

### Pattern 2: anyhow for Application Code

**Evidence from Codex:**
```rust
// codex-rs/utils/pty/src/lib.rs
use anyhow::Result;

pub fn spawn_pty(config: PtyConfig) -> Result<PtyProcess> {
    if config.program.is_none() {
        anyhow::bail!("missing program for PTY spawn");
    }
    // ...
}
```

**Pattern:**
- `anyhow::Result<T>` for application-level functions
- `anyhow::bail!()` for quick error returns
- Context chains with `.context()`
- No custom error types needed

**Usage:** `codex-rs/utils/pty/`, `codex-rs/stdio-to-uds/tests/`

---

## Current NetToolsKit State

### ✅ Already Using anyhow (Correct)

#### 1. core/src/lib.rs
```rust
pub type Result<T> = anyhow::Result<T>;
```
**Status:** ✅ Correct (application-level crate)

#### 2. otel/src/tracing_setup.rs
```rust
use anyhow::Result;
```
**Status:** ✅ Correct (infrastructure crate)

#### 3. ollama/src/client.rs
```rust
use anyhow::Result;
```
**Status:** ✅ Correct (client integration crate)

#### 4. file-search/src/search.rs
```rust
use anyhow::Result;
```
**Status:** ✅ Correct (utility crate)

---

### ⚠️ Needs Migration to thiserror

#### commands/src/lib.rs (PRIORITY: HIGH)

**Current:**
```rust
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
```

**Problem:**
- `Box<dyn Error>` is type-erased (loses error type information)
- No structured error variants
- Hard to pattern match on specific errors
- Not following Codex library pattern

**Recommended:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("template error: {0}")]
    Template(#[from] tera::Error),
}

pub type Result<T> = std::result::Result<T, CommandError>;
```

**Benefits:**
- Type-safe error handling
- Pattern matching on error variants
- Better error messages
- Matches Codex library pattern (core/src/error.rs)

---

## Codex Alignment Score

### Current Alignment: 60%

| Crate | Current | Codex Pattern | Alignment |
|-------|---------|---------------|-----------|
| core | `anyhow::Result` | ✅ anyhow (app) | ✅ 100% |
| otel | `anyhow::Result` | ✅ anyhow (infra) | ✅ 100% |
| ollama | `anyhow::Result` | ✅ anyhow (client) | ✅ 100% |
| file-search | `anyhow::Result` | ✅ anyhow (util) | ✅ 100% |
| **commands** | `Box<dyn Error>` | ❌ thiserror (lib) | ❌ 0% |

**Target Alignment: 100%**

---

## Implementation Plan

### Step 1: Add Dependencies
```toml
# commands/Cargo.toml
[dependencies]
thiserror = "1.0"
```

**Validation:** Codex uses thiserror 1.0.x across all library crates

---

### Step 2: Create CommandError Enum

**File:** `commands/src/error.rs` (new file)

```rust
use thiserror::Error;

/// Errors that can occur during command processing and execution.
#[derive(Error, Debug)]
pub enum CommandError {
    /// Template file not found
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    /// Invalid command syntax or arguments
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    /// Command execution failed
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// Template rendering error
    #[error("template rendering failed: {0}")]
    TemplateError(#[from] tera::Error),

    /// File system error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Async runtime error
    #[error("runtime error: {0}")]
    Runtime(String),
}

pub type Result<T> = std::result::Result<T, CommandError>;
```

**Pattern:** Matches `codex-rs/core/src/error.rs` structure

---

### Step 3: Update commands/src/lib.rs

**Before:**
```rust
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
```

**After:**
```rust
mod error;
pub use error::{CommandError, Result};
```

---

### Step 4: Update Module Imports

**Files to update:**
- `commands/src/processor.rs`
- `commands/src/processor_async.rs`
- `commands/src/apply.rs`
- `commands/src/check.rs`
- `commands/src/list.rs`
- `commands/src/new.rs`
- `commands/src/render.rs`
- `commands/src/async_executor.rs`

**Change:**
```rust
// Before
use crate::{Error, Result};

// After
use crate::{CommandError, Result};
```

---

### Step 5: Update Error Construction

**Before:**
```rust
return Err(format!("template not found: {}", name).into());
```

**After:**
```rust
return Err(CommandError::TemplateNotFound(name.to_string()));
```

---

### Step 6: Update cli Crate (Caller)

**File:** `cli/src/lib.rs`

**Before:**
```rust
use nettoolskit_commands::processor::process_command;

match process_command(&cmd).await {
    Ok(output) => println!("{}", output),
    Err(e) => eprintln!("Error: {}", e),
}
```

**After (with pattern matching):**
```rust
use nettoolskit_commands::{processor::process_command, CommandError};

match process_command(&cmd).await {
    Ok(output) => println!("{}", output),
    Err(CommandError::TemplateNotFound(name)) => {
        eprintln!("Template '{}' not found. Try /list to see available templates.", name);
    },
    Err(CommandError::InvalidCommand(msg)) => {
        eprintln!("Invalid command: {}", msg);
    },
    Err(e) => eprintln!("Error: {}", e),
}
```

**Benefits:**
- Type-safe error handling
- Better UX with specific error messages
- Easier debugging

---

## Validation Checklist

- [ ] Add `thiserror` dependency to commands/Cargo.toml
- [ ] Create `commands/src/error.rs` with CommandError enum
- [ ] Update `commands/src/lib.rs` to export CommandError
- [ ] Update all module imports (8 files)
- [ ] Replace `Box<dyn Error>` construction with CommandError variants
- [ ] Update error handling in `cli/src/lib.rs`
- [ ] Run `cargo build --workspace` (must succeed)
- [ ] Run `cargo test --workspace --lib` (must pass 13/13)
- [ ] Run `cargo clippy --all -- -D warnings` (must be clean)
- [ ] Verify error messages are descriptive and helpful

---

## Expected Benefits

### 1. Type Safety
- Compile-time error variant checking
- Pattern matching on error types
- No runtime type erasure

### 2. Better Error Messages
```rust
// Before: "error: template not found: dotnet"
// After:  "template not found: dotnet"
//         (structured, can add context: "Try /list to see available templates")
```

### 3. Debugging
- Stack traces preserved
- Error chains with `.source()`
- Structured error data

### 4. Codex Alignment
- Matches library error pattern (core/src/error.rs)
- Matches application pattern (utils/pty/)
- 100% alignment with Codex standards

---

## Estimated Effort

**Time:** 1-2 hours

**Breakdown:**
- Create error.rs: 15 min
- Update imports: 15 min
- Update error construction: 30 min
- Update cli error handling: 15 min
- Testing and validation: 15 min

**Complexity:** LOW (straightforward refactoring)

---

## Risks and Mitigation

### Risk 1: Breaking API Changes
**Impact:** External callers using `commands` crate need updates
**Mitigation:** Commands is internal workspace crate, only `cli` uses it
**Status:** LOW RISK

### Risk 2: Test Failures
**Impact:** Tests may expect `Box<dyn Error>` type
**Mitigation:** Update test assertions to match new error type
**Status:** LOW RISK (only 13 tests total)

### Risk 3: Error Conversion Issues
**Impact:** Some errors may not map cleanly to new variants
**Mitigation:** Add `#[from]` attributes for automatic conversion
**Status:** LOW RISK (thiserror handles most cases)

---

## Success Criteria

✅ **Phase 7 Complete When:**
1. All `Box<dyn Error>` replaced with `CommandError` in commands crate
2. `cargo build --workspace` succeeds with 0 warnings
3. `cargo test --workspace --lib` passes 13/13 tests
4. `cargo clippy` produces 0 warnings
5. Error messages are descriptive and user-friendly
6. Code matches Codex error handling patterns (100% alignment)

---

## References
- Codex error pattern: `codex-rs/core/src/error.rs`
- Codex thiserror usage: `codex-rs/git-tooling/src/errors.rs`
- Codex anyhow usage: `codex-rs/utils/pty/src/lib.rs`
- Phase 7 plan: `.docs/cleanup/phase5-roadmap.md`