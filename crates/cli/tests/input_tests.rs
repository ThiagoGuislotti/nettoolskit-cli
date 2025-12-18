//! Input Handling Tests
//!
//! Tests for CLI input processing, including command parsing, text input,
//! and keyboard event handling.

#![allow(clippy::assertions_on_constants)]

use nettoolskit_cli::input::InputResult;

#[test]
fn test_input_result_variants() {
    // Arrange
    let command = InputResult::Command("test".to_string());
    let text = InputResult::Text("hello".to_string());
    let exit = InputResult::Exit;
    let show_menu = InputResult::ShowMenu;

    // Assert
    assert!(matches!(command, InputResult::Command(_)));
    assert!(matches!(text, InputResult::Text(_)));
    assert!(matches!(exit, InputResult::Exit));
    assert!(matches!(show_menu, InputResult::ShowMenu));
}

#[test]
fn test_input_result_debug() {
    // Arrange & Act
    let command = InputResult::Command("list".to_string());
    let debug_str = format!("{:?}", command);

    // Assert
    assert!(debug_str.contains("Command"));
    assert!(debug_str.contains("list"));

    // Arrange & Act
    let text = InputResult::Text("hello world".to_string());
    let debug_str = format!("{:?}", text);

    // Assert
    assert!(debug_str.contains("Text"));
    assert!(debug_str.contains("hello world"));

    // Arrange & Act
    let exit = InputResult::Exit;
    let debug_str = format!("{:?}", exit);

    // Assert
    assert!(debug_str.contains("Exit"));

    // Arrange & Act
    let show_menu = InputResult::ShowMenu;
    let debug_str = format!("{:?}", show_menu);

    // Assert
    assert!(debug_str.contains("ShowMenu"));
}

#[test]
fn test_input_result_pattern_matching() {
    // Arrange
    let results = vec![
        InputResult::Command("/list".to_string()),
        InputResult::Text("regular text".to_string()),
        InputResult::Exit,
        InputResult::ShowMenu,
    ];

    // Act & Assert
    // Test ensures exhaustive pattern matching for all variants
    for result in results {
        match result {
            InputResult::Command(cmd) => {
                assert!(cmd.starts_with("/"));
            }
            InputResult::Text(text) => {
                assert!(!text.is_empty());
            }
            InputResult::Exit => {
                assert!(true);
            }
            InputResult::ShowMenu => {
                assert!(true);
            }
        }
    }
}

#[test]
fn test_input_result_command_extraction() {
    // Arrange
    let command = InputResult::Command("/help".to_string());

    // Act & Assert
    if let InputResult::Command(cmd) = command {
        assert_eq!(cmd, "/help");
    } else {
        panic!("Expected Command variant");
    }
}

#[test]
fn test_input_result_text_extraction() {
    // Arrange
    let text = InputResult::Text("user input".to_string());

    // Act & Assert
    if let InputResult::Text(content) = text {
        assert_eq!(content, "user input");
    } else {
        panic!("Expected Text variant");
    }
}

#[test]
fn test_input_result_empty_strings() {
    // Arrange
    let empty_command = InputResult::Command(String::new());
    let empty_text = InputResult::Text(String::new());

    // Assert
    // Should handle empty strings without panicking
    assert!(matches!(empty_command, InputResult::Command(_)));
    assert!(matches!(empty_text, InputResult::Text(_)));

    // Act & Assert
    if let InputResult::Command(cmd) = empty_command {
        assert!(cmd.is_empty());
    }

    if let InputResult::Text(text) = empty_text {
        assert!(text.is_empty());
    }
}

#[test]
fn test_input_result_string_ownership() {
    // Arrange
    let owned_string = "test command".to_string();

    // Act
    let command = InputResult::Command(owned_string);

    // Assert
    // Should take ownership of the string
    match command {
        InputResult::Command(cmd) => {
            assert_eq!(cmd, "test command");
            // String is now owned by the variant
        }
        _ => panic!("Expected Command"),
    }
}

#[test]
fn test_input_result_all_variants_handled() {
    // Arrange
    // Ensure all variants are covered in pattern matching
    fn handle_result(result: InputResult) -> &'static str {
        match result {
            InputResult::Command(_) => "command",
            InputResult::Text(_) => "text",
            InputResult::Exit => "exit",
            InputResult::ShowMenu => "show_menu",
        }
    }

    // Act & Assert
    assert_eq!(
        handle_result(InputResult::Command("test".to_string())),
        "command"
    );
    assert_eq!(handle_result(InputResult::Text("test".to_string())), "text");
    assert_eq!(handle_result(InputResult::Exit), "exit");
    assert_eq!(handle_result(InputResult::ShowMenu), "show_menu");
}

#[test]
fn test_input_result_clone_behavior() {
    // Arrange
    // Test that we can move InputResult values
    let create_command = || InputResult::Command("test".to_string());
    let create_text = || InputResult::Text("test".to_string());
    let create_exit = || InputResult::Exit;
    let create_show_menu = || InputResult::ShowMenu;

    let cmd = create_command();
    let txt = create_text();
    let ext = create_exit();
    let menu = create_show_menu();

    // Act
    // Should be able to move these values
    let _moved_cmd = cmd;
    let _moved_txt = txt;
    let _moved_ext = ext;
    let _moved_menu = menu;

    // Assert
    assert!(true);
}

#[test]
fn test_input_result_size_efficiency() {
    // Arrange
    use std::mem;

    // Act
    // Test that InputResult doesn't take too much memory
    let size = mem::size_of::<InputResult>();

    // Assert
    // Should be reasonable size (String + discriminant)
    // Exact size depends on platform, but should be < 100 bytes
    assert!(size < 100);
}

#[tokio::test]
async fn test_input_module_integration() {
    // Arrange
    // Test that input module integrates well with async context
    use std::time::Duration;

    // Act
    // Should be able to work in async context
    tokio::time::sleep(Duration::from_millis(1)).await;

    let _result = InputResult::Command("/test".to_string());
    assert!(true);
}

#[test]
fn test_show_menu_variant() {
    // Arrange
    let show_menu = InputResult::ShowMenu;

    // Act & Assert
    // ShowMenu is a unit variant (no data)
    assert!(matches!(show_menu, InputResult::ShowMenu));

    // Pattern already validated above; keep test focused and warning-free.
}

#[test]
fn test_show_menu_in_collection() {
    // Arrange
    let results = [
        InputResult::ShowMenu,
        InputResult::Command("/help".to_string()),
        InputResult::ShowMenu,
        InputResult::Exit,
    ];

    // Act
    let show_menu_count = results
        .iter()
        .filter(|r| matches!(r, InputResult::ShowMenu))
        .count();

    // Assert
    assert_eq!(show_menu_count, 2);
}
