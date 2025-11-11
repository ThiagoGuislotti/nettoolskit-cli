/// Integration tests for CommandRegistry
///
/// Tests the dynamic command registration and execution system,
/// validating async handler invocation, error handling, and registry operations.

use nettoolskit_commands::{CommandRegistry, ExitStatus, Result};

// Helper handlers for testing
async fn success_handler(_args: Vec<String>) -> Result<ExitStatus> {
    Ok(ExitStatus::Success)
}

async fn error_handler(_args: Vec<String>) -> Result<ExitStatus> {
    Ok(ExitStatus::Error)
}

async fn args_echo_handler(args: Vec<String>) -> Result<ExitStatus> {
    if args.is_empty() {
        Ok(ExitStatus::Error)
    } else {
        Ok(ExitStatus::Success)
    }
}

// Registry Creation Tests

#[tokio::test]
async fn test_registry_new_is_empty() {
    // Act
    let registry = CommandRegistry::new();

    // Assert
    assert_eq!(registry.commands().len(), 0);
    assert!(!registry.has_command("any"));
}

#[tokio::test]
async fn test_registry_default_is_empty() {
    // Act
    let registry = CommandRegistry::default();

    // Assert
    assert_eq!(registry.commands().len(), 0);
}

// Command Registration Tests

#[tokio::test]
async fn test_register_single_command() {
    // Arrange
    let mut registry = CommandRegistry::new();

    // Act
    registry.register("/test", success_handler);

    // Assert
    assert!(registry.has_command("/test"));
    assert_eq!(registry.commands().len(), 1);
}

#[tokio::test]
async fn test_register_multiple_commands() {
    // Arrange
    let mut registry = CommandRegistry::new();

    // Act
    registry.register("/cmd1", success_handler);
    registry.register("/cmd2", error_handler);
    registry.register("/cmd3", args_echo_handler);

    // Assert
    assert_eq!(registry.commands().len(), 3);
    assert!(registry.has_command("/cmd1"));
    assert!(registry.has_command("/cmd2"));
    assert!(registry.has_command("/cmd3"));
}

// Command Execution Tests

#[tokio::test]
async fn test_execute_success_command() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/success", success_handler);

    // Act
    let result = registry.execute("/success", vec![]).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Success);
}

#[tokio::test]
async fn test_execute_error_command() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/error", error_handler);

    // Act
    let result = registry.execute("/error", vec![]).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Error);
}

#[tokio::test]
async fn test_execute_unknown_command() {
    // Arrange
    let registry = CommandRegistry::new();

    // Act
    let result = registry.execute("/unknown", vec![]).await;

    // Assert
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unknown command"));
    assert!(error_msg.contains("/unknown"));
}

#[tokio::test]
async fn test_execute_with_args() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/echo", args_echo_handler);

    // Act & Assert - With args should succeed
    let result = registry.execute("/echo", vec!["arg1".to_string()]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Success);

    // Act & Assert - Without args should fail
    let result = registry.execute("/echo", vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Error);
}

// Query Tests

#[tokio::test]
async fn test_commands_list() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/list", success_handler);
    registry.register("/new", success_handler);
    registry.register("/apply", success_handler);

    // Act
    let commands = registry.commands();

    // Assert
    assert_eq!(commands.len(), 3);
    assert!(commands.contains(&"/list".to_string()));
    assert!(commands.contains(&"/new".to_string()));
    assert!(commands.contains(&"/apply".to_string()));
}

#[tokio::test]
async fn test_has_command_case_sensitive() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/Test", success_handler);

    // Assert
    assert!(registry.has_command("/Test"));
    assert!(!registry.has_command("/test"));
    assert!(!registry.has_command("/TEST"));
}

#[tokio::test]
async fn test_register_overwrites_existing() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/test", success_handler);

    // Act
    registry.register("/test", error_handler);
    let result = registry.execute("/test", vec![]).await;

    // Assert
    // Critical: second handler should overwrite the first
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Error);
}

// Concurrent Execution Tests

#[tokio::test]
async fn test_concurrent_execution() {
    // Arrange
    let mut registry = CommandRegistry::new();
    registry.register("/cmd1", success_handler);
    registry.register("/cmd2", error_handler);

    let registry = std::sync::Arc::new(registry);
    let r1 = registry.clone();
    let r2 = registry.clone();

    // Act
    let handle1 = tokio::spawn(async move {
        r1.execute("/cmd1", vec![]).await
    });

    let handle2 = tokio::spawn(async move {
        r2.execute("/cmd2", vec![]).await
    });

    let (result1, result2) = tokio::join!(handle1, handle2);

    // Assert
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().unwrap(), ExitStatus::Success);

    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().unwrap(), ExitStatus::Error);
}

// Advanced Handler Tests

#[tokio::test]
async fn test_closure_handler() {
    // Arrange
    let mut registry = CommandRegistry::new();

    registry.register("/closure", |_args| async {
        Ok(ExitStatus::Success)
    });

    // Act
    let result = registry.execute("/closure", vec![]).await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ExitStatus::Success);
}

#[tokio::test]
async fn test_stateful_handler() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let mut registry = CommandRegistry::new();

    let counter_clone = counter.clone();
    registry.register("/counter", move |_args| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(ExitStatus::Success)
        }
    });

    // Act
    registry.execute("/counter", vec![]).await.unwrap();
    registry.execute("/counter", vec![]).await.unwrap();
    registry.execute("/counter", vec![]).await.unwrap();

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}