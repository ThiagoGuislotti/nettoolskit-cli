# Fix Plan: Cursor Position Bug

**Date**: 2025-11-03
**Issue**: Cursor returns to top after command execution
**Root Cause**: CommandPalette.close() moves cursor to old y_input

---

## Confirmed Root Cause

After detailed code analysis, I confirmed the problem occurs in:

### 1. CommandPalette.close() (MAIN CAUSE)
**File**: `ui/src/legacy/palette.rs` (line 104)

```rust
pub fn close(&mut self) -> io::Result<()> {
    // ...
    if let Ok((x, _)) = cursor::position() {
        queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;  // ❌ PROBLEM
    }
    // ...
}
```

**Problem:**
- `self.y_input` is captured when palette is **opened** (line 70)
- After executing command with large output, `y_input` is **outdated**
- Cursor is moved back to old line (could be top)

### 2. Tui::enter() / Tui::exit() (NOT A CAUSE)
**File**: `ui/src/modern/tui.rs`

```rust
pub fn enter(&mut self) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnableMouseCapture)?;
    // ✅ Does NOT move cursor, only enables raw mode
}

pub fn exit(&mut self) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), DisableMouseCapture)?;
    // ✅ Does NOT move cursor, only disables raw mode
}
```

**Conclusion**: `Tui` is NOT the cause of the problem.

---

## Proposed Solution

### Fix #1: CommandPalette.close() - DO NOT move cursor
**File**: `ui/src/legacy/palette.rs`

**Change:**
```rust
pub fn close(&mut self) -> io::Result<()> {
    if !self.active {
        return Ok(());
    }

    self.active = false;

    // Clear the entire region used by the palette
    self.clear_region()?;

    // ✅ FIX: Only clear region, DO NOT move cursor
    // Cursor is already at correct position after clear_region()
    // Removed: queue!(io::stdout(), cursor::MoveTo(x, self.y_input))?;

    io::stdout().flush()
}
```

**Rationale:**
- `clear_region()` already clears palette lines
- Cursor should stay at **CURRENT** input line, not return to old `y_input`
- After executing command, prompt should appear **below output**, not at top

### Fix #2: clear_region() - Preserve cursor at input line
**File**: `ui/src/legacy/palette.rs` (lines 368-382)

**Change:**
```rust
fn clear_region(&self) -> io::Result<()> {
    // ✅ FIX: Save current cursor position
    let current_pos = cursor::position().unwrap_or((0, self.y_input));
    
    // Clear from line y_input + 1 to end of palette area
    queue!(
        io::stdout(),
        cursor::MoveTo(0, self.y_input + 1),
        terminal::Clear(ClearType::FromCursorDown)
    )?;

    // Clear only necessary lines
    let visible_items = Self::MAX_VISIBLE_ITEMS.min(self.matches.len());
    for i in 0..=visible_items + 1 {
        queue!(
            io::stdout(),
            cursor::MoveTo(0, self.y_input + 1 + i as u16),
            terminal::Clear(terminal::ClearType::CurrentLine)
        )?;
    }

    // ✅ FIX: Restore cursor to CURRENT position, not old y_input
    queue!(io::stdout(), cursor::MoveTo(current_pos.0, current_pos.1))?;

    io::stdout().flush()
}
```

---

## Implementation

### Step 1: Fix palette.close()
- Remove line that moves cursor to y_input
- Cursor should stay where it is

### Step 2: Fix clear_region()
- Save current position before clearing
- Restore current position after clearing
- DO NOT return to old y_input

### Step 3: Test in all modes
- ✅ Legacy mode
- ✅ Modern mode (polling)
- ✅ Modern mode (event stream)

---

## Validation Tests

### Test 1: Command with small output
```
> /health
✅ System OK
> _  ← cursor here (line below output)
```

### Test 2: Command with large output (>10 lines)
```
> /list
Template 1
Template 2
...
Template 20
> _  ← cursor here (line below output, not at top)
```

### Test 3: Multiple sequential commands
```
> /health
✅ System OK
> /list
Template 1...
> /check
✅ Valid
> _  ← cursor always below last output
```

---

## Impact

### Modified Files
- `ui/src/legacy/palette.rs` - 2 functions (`close()`, `clear_region()`)

### Compatibility
- ✅ Does not break public API
- ✅ Does not affect other components
- ✅ Improves UX in all modes

### Risk
- 🟢 **LOW**: Localized change, easy to revert

---

## Next Steps

1. ⬜ Implement fix in `palette.rs`
2. ⬜ Test in legacy mode
3. ⬜ Test in modern mode (polling + stream)
4. ⬜ Validate with various commands (/list, /health, /check)
5. ⬜ Commit with descriptive message
6. ⬜ Update documentation

---

**Status**: 📋 PLAN APPROVED - Ready for implementation
**Priority**: ⭐⭐⭐⭐⭐ CRITICAL
**Estimated Time**: 30 minutes