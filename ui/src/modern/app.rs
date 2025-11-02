/// Application state for the modern TUI
///
/// This structure manages the state of the interactive TUI application,
/// including command history, current input, execution state, and visual feedback.

/// Main application state for the modern TUI
pub struct App {
    /// Current user input in the command line
    pub input: String,
    /// Command history (most recent first)
    pub history: Vec<String>,
    /// Current position in history navigation
    pub history_index: Option<usize>,
    /// Whether the application should exit
    pub should_quit: bool,
    /// Current execution state
    pub execution_state: ExecutionState,
    /// Status message to display
    pub status_message: Option<String>,
}

/// Execution state of the application
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// Ready to accept commands
    Idle,
    /// Executing a command
    Running,
    /// Command completed successfully
    Success,
    /// Command failed with error
    Error(String),
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            history_index: None,
            should_quit: false,
            execution_state: ExecutionState::Idle,
            status_message: None,
        }
    }

    /// Handle character input
    pub fn on_char(&mut self, c: char) {
        self.input.push(c);
        self.history_index = None; // Reset history navigation
    }

    /// Handle backspace
    pub fn on_backspace(&mut self) {
        self.input.pop();
        self.history_index = None;
    }

    /// Handle command submission
    pub fn on_submit(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return None;
        }

        let command = self.input.clone();

        // Add to history (avoid duplicates)
        if self.history.first() != Some(&command) {
            self.history.insert(0, command.clone());
            // Keep history manageable (max 100 commands)
            self.history.truncate(100);
        }

        self.input.clear();
        self.history_index = None;

        Some(command)
    }

    /// Navigate history up (older commands)
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => 0,
            Some(i) if i + 1 < self.history.len() => i + 1,
            Some(i) => i,
        };

        self.history_index = Some(new_index);
        self.input = self.history[new_index].clone();
    }

    /// Navigate history down (newer commands)
    pub fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(0) => {
                self.history_index = None;
                self.input.clear();
            }
            Some(i) => {
                let new_index = i - 1;
                self.history_index = Some(new_index);
                self.input = self.history[new_index].clone();
            }
        }
    }

    /// Mark that the application should quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Set execution state
    pub fn set_execution_state(&mut self, state: ExecutionState) {
        self.execution_state = state;
    }

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}