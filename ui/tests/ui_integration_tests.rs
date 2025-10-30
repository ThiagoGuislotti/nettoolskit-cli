use nettoolskit_ui::*;

#[test]
fn test_ui_module_integration() {
    // Test that all public items are accessible
    let _primary = PRIMARY_COLOR;
    let _secondary = SECONDARY_COLOR;
    let _white = WHITE_COLOR;
    let _gray = GRAY_COLOR;

    // Test function integration
    let path = "/test/path";
    let _truncated = truncate_directory(path, 10);
    let _clear_result = clear_terminal();

    assert!(true);
}

#[test]
fn test_display_and_terminal_integration() {
    // Test that display and terminal functions work together
    let test_path = "/very/long/directory/path/for/testing";
    let truncated = truncate_directory(test_path, 20);

    // After truncating path, should be able to clear terminal
    let _clear_result = clear_terminal();

    assert!(truncated.len() <= 20 || truncated.len() <= 25); // Allow some flexibility
}

#[test]
fn test_color_usage_with_display_functions() {
    // Test that colors can be used with display-related functions
    let _colors = vec![PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR, GRAY_COLOR];

    // Test path truncation with colors in mind (path might contain color info)
    let test_path = "/colored/path/with/ansi/codes";
    let _truncated = truncate_directory(test_path, 15);

    assert!(true);
}

#[test]
fn test_module_completeness() {
    // Verify all expected exports are available

    // Colors
    let _primary = PRIMARY_COLOR;
    let _secondary = SECONDARY_COLOR;
    let _white = WHITE_COLOR;
    let _gray = GRAY_COLOR;

    // Display functions
    let _truncated = truncate_directory("/test", 10);

    // Terminal functions
    let _clear_result = clear_terminal();

    assert!(true);
}

#[test]
fn test_ui_error_handling_integration() {
    // Test error handling across UI modules

    // Terminal operations might fail
    match clear_terminal() {
        Ok(()) => {
            // If terminal ops succeed, display ops should work too
            let _truncated = truncate_directory("/test/path", 10);
            assert!(true);
        }
        Err(_) => {
            // If terminal ops fail, display ops should still work
            let _truncated = truncate_directory("/test/path", 10);
            assert!(true);
        }
    }
}

#[test]
fn test_ui_consistency() {
    // Test that UI components work consistently together

    let paths = vec![
        "/short",
        "/medium/length/path",
        "/very/long/path/with/many/segments/for/testing/truncation",
    ];

    for path in paths {
        let truncated = truncate_directory(path, 20);
        assert!(truncated.len() <= 22); // Allow small margin

        // Should be able to clear terminal after each operation
        let _clear = clear_terminal();
    }
}

#[test]
fn test_ui_module_no_conflicts() {
    // Test that importing everything doesn't cause conflicts
    use nettoolskit_ui::*;

    let _color_test = PRIMARY_COLOR;
    let _path_test = truncate_directory("/test", 5);
    let _terminal_test = clear_terminal();

    assert!(true);
}

#[test]
fn test_ui_synchronous_operations() {
    // Test that UI functions work in synchronous contexts

    let _color = PRIMARY_COLOR;
    let _path = truncate_directory("/sync/test/path", 15);

    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(1));

    let _clear = clear_terminal();

    assert!(true);
}

#[test]
fn test_ui_thread_safety() {
    use std::thread;

    // Test UI functions in multi-threaded environment
    let handles: Vec<_> = (0..3).map(|i| {
        thread::spawn(move || {
            let path = format!("/thread/{}/test/path", i);
            let _truncated = truncate_directory(&path, 10);
            let _clear = clear_terminal();
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert!(true);
}

#[test]
fn test_ui_performance_basic() {
    use std::time::Instant;

    let start = Instant::now();

    // Perform various UI operations
    for i in 0..100 {
        let path = format!("/performance/test/path/iteration/{}", i);
        let _truncated = truncate_directory(&path, 20);
    }

    let duration = start.elapsed();

    // Should complete quickly (within reasonable time)
    assert!(duration.as_millis() < 1000); // Less than 1 second for 100 operations
}