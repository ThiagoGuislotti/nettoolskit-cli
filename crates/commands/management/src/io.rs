/// I/O utilities for CLI output formatting
use owo_colors::OwoColorize;

/// Exit status for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// Command executed successfully
    Success,
    /// Command encountered an error
    Error,
}

/// Trait for formatting CLI output
pub trait OutputFormatter {
    /// Display an informational message
    fn info(&mut self, message: &str);

    /// Display a success message
    fn success(&mut self, message: &str);

    /// Display a warning message
    fn warning(&mut self, message: &str);

    /// Display an error message
    fn error(&mut self, message: &str);

    /// Display a section header
    fn section(&mut self, title: &str);

    /// Add a blank line
    fn blank_line(&mut self);
}

/// Terminal output formatter with colors
pub struct TerminalOutputFormatter;

impl TerminalOutputFormatter {
    /// Create a new terminal output formatter
    pub fn new() -> Self {
        Self
    }
}

impl Default for TerminalOutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for TerminalOutputFormatter {
    fn info(&mut self, message: &str) {
        println!("{}", message);
    }

    fn success(&mut self, message: &str) {
        println!("{}", message.green());
    }

    fn warning(&mut self, message: &str) {
        println!("{}", message.yellow());
    }

    fn error(&mut self, message: &str) {
        println!("{}", message.red().bold());
    }

    fn section(&mut self, title: &str) {
        println!("\n{}", title.cyan().bold());
        println!("{}", "─".repeat(title.len()).cyan());
    }

    fn blank_line(&mut self) {
        println!();
    }
}

/// Mock output formatter for testing
#[cfg(test)]
pub struct MockOutputFormatter {
    pub messages: Vec<String>,
    errors: Vec<String>,
}

#[cfg(test)]
impl MockOutputFormatter {
    /// Create a new mock formatter
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check if any errors were recorded
    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
impl Default for MockOutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl OutputFormatter for MockOutputFormatter {
    fn info(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    fn success(&mut self, message: &str) {
        self.messages.push(format!("SUCCESS: {}", message));
    }

    fn warning(&mut self, message: &str) {
        self.messages.push(format!("WARNING: {}", message));
    }

    fn error(&mut self, message: &str) {
        self.errors.push(message.to_string());
        self.messages.push(format!("ERROR: {}", message));
    }

    fn section(&mut self, title: &str) {
        self.messages.push(format!("SECTION: {}", title));
    }

    fn blank_line(&mut self) {
        self.messages.push(String::new());
    }
}
