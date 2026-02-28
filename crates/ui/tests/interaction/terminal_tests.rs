// Terminal module tests - public API tests
use nettoolskit_ui::{begin_interactive_logging, disable_interactive_logging};
use serial_test::serial;

#[test]
#[serial]
fn test_begin_interactive_logging_returns_guard() {
    let guard = begin_interactive_logging();
    drop(guard);
}

#[test]
#[serial]
fn test_interactive_log_guard_deactivate() {
    let mut guard = begin_interactive_logging();
    guard.deactivate();
    // Second deactivate should be a no-op
    guard.deactivate();
    drop(guard);
}

#[test]
#[serial]
fn test_disable_interactive_logging_without_enable() {
    // Calling disable without a prior enable should not panic
    disable_interactive_logging();
}

#[test]
#[serial]
fn test_interactive_logging_enable_disable_cycle() {
    let guard = begin_interactive_logging();
    drop(guard); // drop should disable

    // Ensure we can start again after drop
    let guard2 = begin_interactive_logging();
    drop(guard2);
}

#[test]
#[serial]
fn test_interactive_logging_guard_drop_disables() {
    {
        let _guard = begin_interactive_logging();
        // Guard is dropped here
    }
    // Should be safe to call disable again
    disable_interactive_logging();
}

#[test]
#[serial]
fn test_interactive_logging_deactivate_then_drop() {
    let mut guard = begin_interactive_logging();
    guard.deactivate();
    // drop after deactivate should not double-disable
    drop(guard);
}

#[test]
#[serial]
fn test_multiple_sequential_sessions() {
    for _ in 0..5 {
        let guard = begin_interactive_logging();
        drop(guard);
    }
}

#[test]
#[serial]
fn test_disable_is_idempotent() {
    disable_interactive_logging();
    disable_interactive_logging();
    disable_interactive_logging();
}
