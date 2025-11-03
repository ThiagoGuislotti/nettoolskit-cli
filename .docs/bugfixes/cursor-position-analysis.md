# Critical Analysis: Cursor Position Bug After Command Execution

**Date**: 2025-11-03
**Status**: 🔴 CRITICAL - Cursor returns to top after command execution
**Priority**: HIGH
**Affected Phase**: All (Legacy, Modern Polling, Modern Stream)

---

## 1. Problem Description

### Current Behavior (❌ INCORRECT)
When a command is executed (e.g., `/list`, `/check`), the cursor **returns to the top of the terminal** after displaying results, forcing users to manually scroll to see the input prompt.

**Problematic Flow:**
```
1. User types: /list
2. Command palette closes
3. Command executes
4. Output is displayed
5. ❌ PROBLEM: Cursor returns to line 0 (top)
6. Prompt "> " appears at top (invisible if output is large)
```

### Expected Behavior (✅ CORRECT)
The cursor should **remain below the last output**, skipping only one line:

```
1. User types: /list
2. Command palette closes
3. Command executes
4. Output is displayed
5. ✅ CORRECT: Cursor stays below output
6. New blank line
7. Prompt "> " appears right below output
```

---

## 2. Code Analysis

### 2.1 Modern Mode (Event Stream & Polling)
**File**: `cli/src/lib.rs` (lines 230-310)

#### Modern Loop with Stream
```rust
async fn run_modern_loop_with_stream(...) -> io::Result<ExitStatus> {
    // ...
    loop {
        match handle_events_stream(input_buffer, palette, &mut events).await? {
            EventResult::Command(cmd) => {
                tui.exit()?;  // ← Exits raw mode
                
                // Execute command
                let status = if is_async_command(&cmd) {
                    process_async_command(&cmd).await // ← Output here
                } else {
                    process_command(&cmd).await.into()
                };

                // 🔴 PROBLEM: After output, prints prompt BEFORE re-entering raw mode
                print!("\n> ");  // ← Line 279
                std::io::Write::flush(&mut std::io::stdout())?;
                input_buffer.clear();

                tui.enter()?;  // ← Re-enters raw mode
            }
        }
    }
}
```

**Analysis:**
1. ✅ `tui.exit()` restores terminal correctly
2. ✅ Command executes and prints output
3. ❌ **PROBLEM**: `print!("\n> ")` doesn't capture current cursor position
4. ❌ **PROBLEM**: `tui.enter()` may reset cursor to previous saved position

#### Modern Loop with Polling
**File**: `cli/src/lib.rs` (lines 312-387)

Has **exactly the same** problematic pattern:
```rust
// Line 361
print!("\n> ");
std::io::Write::flush(&mut std::io::stdout())?;
input_buffer.clear();

tui.enter()?;
```

### 2.2 Legacy Mode
**File**: `cli/src/lib.rs` (lines 389-450)

```rust
async fn run_legacy_loop(...) -> io::Result<ExitStatus> {
    let mut raw_mode = RawModeGuard::new()?;

    loop {
        raw_mode.enable()?;
        print!("> ");  // ← Initial prompt
        std::io::Write::flush(&mut std::io::stdout())?;
        input_buffer.clear();

        match read_line_with_palette(input_buffer, palette).await? {
            InputResult::Command(cmd) => {
                raw_mode.disable()?;
                
                // Execute command (output here)
                let status: ExitStatus = process_command(&cmd).await.into();
                
                raw_mode.enable()?;
                // NOTE: Layout guard kept for commands
                ensure_layout_guard();  // ← 🔴 SUSPECT #1
            }
            // ...
        }

        println!();  // ← 🔴 SUSPECT #2: Only one new line
    }
}
```

**Analysis:**
1. ✅ Disable/enable raw mode preserved
2. ❌ **PROBLEM #1**: `ensure_layout_guard()` may be moving cursor
3. ❌ **PROBLEM #2**: `println!()` only adds new line but doesn't guarantee visible prompt

### 2.3 Tui Enter/Exit
**File**: `ui/src/modern/tui.rs` (needs verification)

Problem may be in `Tui::enter()` and `Tui::exit()` implementation:
- `enter()`: Does it save cursor position? Move cursor to saved position?
- `exit()`: Where does it restore cursor to?

### 2.4 CommandPalette Close
**File**: `ui/src/legacy/palette.rs` (lines 94-113)

```rust
pub fn close(&mut self) -> io::Result<()> {
    if !self.active {
        return Ok(());
    }

    self.active = false;

    // Clear the entire region used by the palette
    self.clear_region()?;

    // Reposition cursor to input line
    // Do not print additional lines to history
    if let Ok((x, _)) = cursor::position() {
        queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;  // ← 🔴 SUSPECT #3
    }

    io::stdout().flush()
}
```

**Analysis:**
- ❌ **PROBLEM #3**: `cursor::MoveTo(x, self.y_input)` moves cursor to `y_input` (OLD position)
- ❌ After executing command, `y_input` is not updated
- ❌ Cursor returns to line where palette was opened (could be line 0 or top)

### 2.5 Ensure Layout Integrity
**File**: `ui/src/legacy/terminal.rs` (lines 378-393)

```rust
pub fn ensure_layout_integrity() -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap();
        slot.clone()
    };

    if let Some(active) = layout {
        active.ensure_scroll_region()  // ← 🔴 SUSPECT #4
    } else {
        Ok(())
    }
}
```

**Analysis:**
- ❌ **PROBLEM #4**: `ensure_scroll_region()` may reset cursor
- Need to check `ensure_scroll_region()` implementation

---

## 3. Root Cause Hypotheses

### Hypothesis #1: CommandPalette.close() moves cursor to old y_input ⭐⭐⭐⭐⭐
**Probability**: 95%
**Evidence:**
- `palette.close()` moves cursor to `self.y_input` (line where palette was opened)
- After executing command with large output, `y_input` is not updated
- Cursor returns to old position (could be top)

**Test:**
```rust
// In palette.close():
println!("DEBUG: Moving cursor to y_input={}", self.y_input);
```

### Hypothesis #2: Tui::enter() restores cursor to old saved position ⭐⭐⭐⭐
**Probability**: 80%
**Evidence:**
- `tui.enter()` may save/restore cursor position
- After large output, saved position is outdated
- Cursor returns to old position when re-entering raw mode

**Test:**
```rust
// In tui.enter():
let pos = cursor::position()?;
println!("DEBUG: Entering raw mode at position {:?}", pos);
```

### Hypothesis #3: ensure_layout_guard() explicitly moves cursor ⭐⭐⭐
**Probability**: 60%
**Evidence:**
- `ensure_layout_guard()` is only called after commands (not after text)
- May be resetting scroll region and cursor
- Implementation in `terminal.rs` may have side effects

**Test:**
```rust
// In ensure_layout_guard():
println!("DEBUG: Before ensure_layout_integrity");
let pos1 = cursor::position()?;
ensure_layout_integrity()?;
let pos2 = cursor::position()?;
println!("DEBUG: Cursor moved from {:?} to {:?}", pos1, pos2);
```

### Hypothesis #4: Terminal layout footer interferes with cursor ⭐⭐
**Probability**: 40%
**Evidence:**
- Fixed footer uses ANSI scroll region (`CSI r`)
- Scroll region may affect cursor position
- But problem occurs in **all modes** (with and without footer)

---

## 4. Investigation Plan

### Step 1: Instrumentation ✅ TODO
Add debug logging to track cursor position:

```rust
// Macro helper
macro_rules! debug_cursor {
    ($label:expr) => {
        if std::env::var("NTK_DEBUG_CURSOR").is_ok() {
            let pos = crossterm::cursor::position().unwrap_or((0, 0));
            eprintln!("[CURSOR] {}: {:?}", $label, pos);
        }
    };
}

// Critical instrumentation points:
1. Before palette.close()
2. After palette.close()
3. Before tui.exit()
4. After tui.exit()
5. Before process_command()
6. After process_command()
7. Before print!("\n> ")
8. Before tui.enter()
9. After tui.enter()
10. Before/after ensure_layout_guard()
```

### Step 2: Controlled Reproduction ✅ TODO
1. Run CLI with `NTK_DEBUG_CURSOR=1`
2. Type `/list`
3. Observe debug output
4. Identify where cursor "jumps" to wrong position

### Step 3: Code Verification ✅ TODO
Review implementation of:
- [ ] `Tui::enter()` - saves/restores cursor?
- [ ] `Tui::exit()` - restores cursor to where?
- [ ] `TerminalLayoutInner::ensure_scroll_region()` - moves cursor?
- [ ] `CommandPalette::close()` - y_input outdated?

### Step 4: Isolated Tests ✅ TODO
Create unit tests for:
- [ ] Cursor preservation after `tui.enter()/exit()`
- [ ] Cursor position after `palette.close()`
- [ ] Cursor position after `ensure_layout_guard()`

---

## 5. Proposed Solutions

### Solution #1: Capture cursor position AFTER output ⭐⭐⭐⭐⭐
**Priority**: HIGH
**Complexity**: LOW

```rust
// Modern loops (stream & polling)
EventResult::Command(cmd) => {
    tui.exit()?;
    
    // Execute command
    let status = process_command(&cmd).await.into();
    
    // ✅ FIX: Capture CURRENT cursor position (after output)
    let (x, y) = cursor::position().unwrap_or((0, 0));
    
    // ✅ Move cursor to line below output
    print!("\n> ");
    std::io::Write::flush(&mut std::io::stdout())?;
    input_buffer.clear();

    // ✅ Save new position before re-entering
    tui.enter_at(x, y + 1)?;  // New method that preserves position
}
```

**Advantages:**
- ✅ Direct and simple solution
- ✅ Works in all modes
- ✅ Doesn't require changes to `CommandPalette`

**Disadvantages:**
- ⚠️ Requires new method `Tui::enter_at()`

### Solution #2: Update y_input in CommandPalette ⭐⭐⭐⭐
**Priority**: MEDIUM
**Complexity**: LOW

```rust
pub fn close(&mut self) -> io::Result<()> {
    if !self.active {
        return Ok(());
    }

    self.active = false;
    self.clear_region()?;

    // ✅ FIX: Capture CURRENT position instead of using old y_input
    if let Ok((x, y)) = cursor::position() {
        // Just clear region, don't move cursor
        // queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;  // ❌ REMOVED
        queue!(io::stdout(), cursor::MoveTo(x, y))?;  // ✅ Keep current position
    }

    io::stdout().flush()
}
```

**Advantages:**
- ✅ Targeted fix
- ✅ Doesn't affect other components
- ✅ Simple to test

**Disadvantages:**
- ⚠️ `clear_region()` still uses `self.y_input` internally
- ⚠️ May not completely solve the problem

### Solution #3: Remove ensure_layout_guard() after commands ⭐⭐⭐
**Priority**: LOW
**Complexity**: LOW

```rust
// Legacy loop
InputResult::Command(cmd) => {
    raw_mode.disable()?;
    let status: ExitStatus = process_command(&cmd).await.into();
    raw_mode.enable()?;
    // ❌ REMOVED: ensure_layout_guard();
}
```

**Advantages:**
- ✅ Removes possible cause of problem
- ✅ Simplifies code

**Disadvantages:**
- ⚠️ Layout guard may be necessary for commands that alter terminal
- ⚠️ May break layout in some cases

### Solution #4: Redesign cursor tracking logic ⭐⭐
**Priority**: VERY LOW (only if other solutions fail)
**Complexity**: VERY HIGH

Create centralized cursor tracking system:
- Global state `CURSOR_TRACKER`
- Updates position after each operation
- Restores correct position before prompt

**Advantages:**
- ✅ Robust and complete solution
- ✅ Prevents future bugs

**Disadvantages:**
- ❌ High complexity
- ❌ Massive refactoring
- ❌ Risk of regressions

---

## 6. Next Steps

### Immediate (today)
1. ✅ Add instrumentation (debug_cursor macro)
2. ✅ Reproduce problem with logging
3. ✅ Identify exact problem line

### Short Term (1-2 days)
4. ⬜ Implement Solution #1 (capture position after output)
5. ⬜ Implement Solution #2 (fix palette.close())
6. ⬜ Test in all modes (legacy, modern polling, modern stream)
7. ⬜ Validate with different output sizes

### Medium Term (3-5 days)
8. ⬜ Add automated tests
9. ⬜ Document correct behavior
10. ⬜ Update task-phase-2.4 with fix

---

## 7. Impact

### Affected Users
- ✅ **ALL** CLI users
- ✅ **ALL** modes (legacy, modern polling, modern stream)
- ✅ **ALL** commands that generate output

### Severity
- 🔴 **CRITICAL**: UX completely broken
- 🔴 CLI unusable for commands with long output
- 🔴 User must manually scroll after each command

### Priority
- ⭐⭐⭐⭐⭐ **MAXIMUM**: Must be fixed BEFORE Phase 2.4

---

## 8. References

### Related Files
- `cli/src/lib.rs` - Main loops (legacy, modern stream, modern polling)
- `ui/src/modern/tui.rs` - Tui::enter()/exit()
- `ui/src/modern/events.rs` - Event handlers
- `ui/src/legacy/palette.rs` - CommandPalette::close()
- `ui/src/legacy/terminal.rs` - ensure_layout_integrity()

### Related Issues
- `.docs/bugfixes/cursor-position-pending.md` - Known issue since Phase 1.2

### Documentation
- Section 2.3 of `nettoolskit-cli.md` - Terminal Layout Architecture
- FR17: "Ensure input prompt always repositions below latest output (cursor safety)"

---

## 9. Conclusion

The cursor problem is **CRITICAL** and affects the usability of the entire CLI. The most likely root cause is a combination of:

1. **CommandPalette.close()** moves cursor to old `y_input` (position where palette was opened)
2. **Tui::enter()** may restore cursor to old saved position
3. **ensure_layout_guard()** may explicitly move cursor

The most effective solution is to **capture the current cursor position AFTER command output** and ensure the next prompt appears right below, without returning to old positions.

**Recommended action**: Implement Solution #1 + Solution #2 together for maximum robustness.

---

**Status**: 📋 ANALYSIS COMPLETE - Awaiting instrumentation and tests
**Assignee**: @dev-team
**Milestone**: Phase 2.4 (Blocker)