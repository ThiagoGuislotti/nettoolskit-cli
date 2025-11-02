# Bug: Cursor Position After Command (Pending Fix)

**Status**: 🔴 Known Issue - Deferred to later phase
**Priority**: Medium
**Phase**: To be fixed after Phase 1.3

---

## Issue Description

After selecting a command from the palette in modern mode, the cursor/input prompt returns to the top of the terminal instead of staying below the command output.

**Expected**: Prompt `>` appears below command output
**Actual**: Prompt appears at top of terminal

---

## Evidence

See screenshot: User reported issue persists after initial fix attempts.

---

## Attempted Fixes

1. ✅ Added explicit prompt printing after command
2. ✅ Removed `ensure_layout_guard()` from modern mode
3. ❌ Still not working correctly

---

## Root Cause (Hypothesis)

The issue is likely related to how raw mode interacts with terminal cursor positioning. The legacy mode works because it has a different loop structure that naturally resets cursor position.

**Possible causes:**
- Raw mode cursor positioning needs explicit handling
- Ratatui terminal state interfering with cursor
- Need to use crossterm cursor movement commands
- Missing terminal flush at critical point

---

## Next Steps

1. **After Phase 1.3**: Return to this issue
2. Test with explicit cursor positioning using crossterm
3. Compare terminal state between legacy and modern at cursor point
4. Potentially need to track cursor position manually

---

## Workaround

Use legacy mode for now:
```bash
cargo run  # Without modern-tui feature
```

---

## Impact

- Modern mode usability affected
- Not blocking other improvements
- Legacy mode works perfectly
- Can defer fix to later phase

---

**Decision**: Continue with Phase 1.3 improvements, fix cursor positioning later.
