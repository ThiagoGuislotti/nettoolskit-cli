# TUI UX Guidelines

This document defines the official UX contract for terminal interfaces in NetToolsKit CLI.
It is the baseline for feature development, bug fixing, and code reviews touching `crates/ui` and `crates/cli`.

## Goals

- Keep the interface stable during long interactive sessions.
- Preserve readability under terminal resize and font-size changes.
- Provide predictable behavior across Windows Terminal, conhost, and Unix terminals.
- Degrade gracefully on narrow terminals and limited color/unicode capabilities.

## Layout Contract

- Header:
  - Rendered at startup and re-rendered after resize.
  - Contains branding and context, but must not block interaction in narrow mode.
- Dynamic area:
  - Main interactive zone where prompts and menus run.
  - Bound by scroll region management in `crates/ui/src/interaction/terminal.rs`.
- Footer:
  - Dedicated log/status region with bounded history.
  - Must not overlap dynamic area after reconfigure.

## Runtime Invariants

- No alternate screen usage for primary interactive flow.
- Terminal output/history must remain visible on `/quit` and `Ctrl+C`.
- Cursor must remain visible and blinking in prompt-ready states.
- Reconfigure must be idempotent when dimensions do not change.
- Resize processing must be debounced (trailing edge) to prevent redraw storms.

## Breakpoint Rules

- `>= 80 columns` (full mode):
  - Full instructions and descriptive prompts.
  - Subtitle/footer metadata can be shown.
- `60-79 columns` (narrow mode):
  - Shorter instructions and prompt text.
  - Reduce non-essential metadata to protect command readability.
- `< 60 columns` (compact mode):
  - Minimal instruction copy and compact prompt.
  - Keep only critical interaction elements on screen.

## Capability Fallback Rules

- Unicode:
  - Prefer unicode arrows/box-drawing only when capability detection allows.
  - Use ASCII fallback for all critical symbols and borders.
- Colors:
  - Respect color capability detection and `NO_COLOR`/overrides.
  - UI must remain understandable without color (no color-only semantics).

## Resize Behavior

- Resize events are recorded first, then processed by debounce window.
- Reconfigure flow must:
  - hide cursor
  - clear/reflow deterministically
  - restore layout metrics and scroll region
  - restore cursor visibility
- On temporarily invalid viewport (too small):
  - do not panic
  - reset terminal to usable state
  - recover automatically when size becomes valid again

## Error State UX

- Validation/command failures:
  - show clear, short error text
  - avoid stack traces in normal UX path
- Terminal/layout failures:
  - prefer recovery and graceful degradation over hard exit
  - keep shell usable even after runtime interruptions

## Interaction Standards

- Primary menu guidance must include:
  - navigation keys
  - selection key
  - quit command
- Prompt style should stay consistent across modules (`CommandPalette`, enum menus, command handlers).
- Menu labels should remain scannable on narrow width (adaptive alignment).

## Engineering Checklist

Before merging TUI-affecting changes:

1. Validate `cargo fmt --all -- --check`.
2. Validate `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
3. Validate `cargo test --workspace --all-targets`.
4. Validate Rust vulnerability gate (Critical/High blocked).
5. Confirm resize tests include shrink/grow sequences and recovery scenarios.

## Source of Truth

- Runtime mechanics: `crates/ui/src/interaction/terminal.rs`
- Menu rendering: `crates/ui/src/rendering/components/`
- Startup/header behavior: `crates/cli/src/display.rs`
- Operational troubleshooting: `docs/operations/incident-response-playbook.md`