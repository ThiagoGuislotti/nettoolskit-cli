//! Directory utility functions
//!
//! This module provides utilities for working with directories,
//! including path formatting and home directory substitution.

use std::env;

/// Returns the current directory as a string with home directory substitution.
///
/// This function retrieves the current working directory and replaces the home
/// directory path with a tilde (~) for cleaner display. On Windows, it uses
/// USERPROFILE; on Unix-like systems, it uses HOME.
///
/// # Returns
///
/// A String containing the current directory path with ~ substitution.
/// If the current directory cannot be determined, returns the path as-is.
///
/// # Examples
///
/// ```
/// use nettoolskit_core::path_utils::directory::get_current_directory;
///
/// let dir = get_current_directory();
/// println!("Current directory: {}", dir);
/// // On Unix: ~/projects/my-app
/// // On Windows: ~\Documents\projects\my-app
/// ```
pub fn get_current_directory() -> String {
    if let Ok(current_dir) = env::current_dir() {
        let current_dir_str = current_dir.to_string_lossy().to_string();

        // Try to replace home directory with ~
        if let Ok(home) = env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) {
            if let Some(relative) = current_dir_str.strip_prefix(&home) {
                return format!("~{}", relative);
            }
        }

        current_dir_str
    } else {
        String::from(".")
    }
}