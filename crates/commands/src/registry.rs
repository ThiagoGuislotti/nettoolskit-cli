/// Command registry for managing available commands
///
/// This module provides a centralized registry for all CLI commands,
/// enabling dynamic command discovery and execution.
use crate::{CommandError, ExitStatus, Result};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Type alias for async command handler functions
pub type CommandHandler = Box<
    dyn Fn(Vec<String>) -> Pin<Box<dyn Future<Output = Result<ExitStatus>> + Send>> + Send + Sync,
>;

/// Registry for mapping command names to their handlers
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command with its handler
    pub fn register<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(Vec<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ExitStatus>> + Send + 'static,
    {
        let name = name.into();
        let boxed: CommandHandler = Box::new(move |args| Box::pin(handler(args)));
        self.handlers.insert(name, boxed);
    }

    /// Execute a registered command
    pub async fn execute(&self, name: &str, args: Vec<String>) -> Result<ExitStatus> {
        match self.handlers.get(name) {
            Some(handler) => handler(args).await,
            None => Err(CommandError::InvalidCommand(format!(
                "Unknown command: {}",
                name
            ))),
        }
    }

    /// Check if a command is registered
    pub fn has_command(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Get list of all registered command names
    pub fn commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn dummy_handler(_args: Vec<String>) -> Result<ExitStatus> {
        Ok(ExitStatus::Success)
    }

    #[tokio::test]
    async fn test_register_and_execute() {
        let mut registry = CommandRegistry::new();
        registry.register("test", dummy_handler);

        assert!(registry.has_command("test"));
        let result = registry.execute("test", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitStatus::Success);
    }

    #[tokio::test]
    async fn test_unknown_command() {
        let registry = CommandRegistry::new();
        let result = registry.execute("unknown", vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_commands() {
        let mut registry = CommandRegistry::new();
        registry.register("cmd1", dummy_handler);
        registry.register("cmd2", dummy_handler);

        let commands = registry.commands();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"cmd1".to_string()));
        assert!(commands.contains(&"cmd2".to_string()));
    }
}
