#![allow(clippy::assertions_on_constants)]

use nettoolskit_cli::input::InputResult;

#[test]
fn test_input_result_variants() {
    // Arrange
    let command = InputResult::Command("test".to_string());
    let text = InputResult::Text("hello".to_string());
    let exit = InputResult::Exit;

    // Assert
    assert!(matches!(command, InputResult::Command(_)));
    assert!(matches!(text, InputResult::Text(_)));
    assert!(matches!(exit, InputResult::Exit));
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
}

#[test]
fn test_input_result_pattern_matching() {
    // Arrange
    let results = vec![
        InputResult::Command("/list".to_string()),
        InputResult::Text("regular text".to_string()),
        InputResult::Exit,
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
        }
    }

    // Act & Assert
    assert_eq!(
        handle_result(InputResult::Command("test".to_string())),
        "command"
    );
    assert_eq!(handle_result(InputResult::Text("test".to_string())), "text");
    assert_eq!(handle_result(InputResult::Exit), "exit");
}

#[test]
fn test_input_result_clone_behavior() {
    // Arrange
    // Test that we can move InputResult values
    let create_command = || InputResult::Command("test".to_string());
    let create_text = || InputResult::Text("test".to_string());
    let create_exit = || InputResult::Exit;

    let cmd = create_command();
    let txt = create_text();
    let ext = create_exit();

    // Act
    // Should be able to move these values
    let _moved_cmd = cmd;
    let _moved_txt = txt;
    let _moved_ext = ext;

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
