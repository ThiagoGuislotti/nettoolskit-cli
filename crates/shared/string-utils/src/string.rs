//! String utilities for `NetToolsKit` CLI
//!
//! This module provides common string manipulation utilities
//! used throughout the `NetToolsKit` CLI application.

/// Detect directory separator (Windows backslash or Unix forward slash)
fn detect_separator(path: &str) -> char {
    if path.contains('\\') {
        '\\'
    } else {
        '/'
    }
}

/// Create a simple fallback truncation when path components are insufficient
fn simple_truncate(dir: &str, max_width: usize, ellipsis: &str, separator: char) -> String {
    let available_space = max_width.saturating_sub(ellipsis.len());
    let front_length = available_space * 35 / 100;
    let back_length = available_space - front_length;

    let front = &dir[..front_length.min(dir.len())];
    let back_start = dir.len().saturating_sub(back_length);
    let back = &dir[back_start..];

    format!("{front}{ellipsis}{separator}{back}")
}

/// Truncate a directory path to fit within a maximum width.
///
/// This function intelligently truncates long directory paths by preserving
/// the first and last components and using ellipsis in the middle when possible.
/// It handles both Windows and Unix path separators automatically.
///
/// # Arguments
///
/// * `dir` - The directory path to truncate
/// * `max_width` - Maximum width for the result string
///
/// # Returns
///
/// A truncated path string that fits within `max_width` characters
///
/// # Examples
///
/// ```
/// use nettoolskit_string_utils::string::truncate_directory;
/// let truncated = truncate_directory("/very/long/path/to/project", 20);
/// // Returns something like "/very/.../project"
/// ```
#[must_use]
pub fn truncate_directory(dir: &str, max_width: usize) -> String {
    if dir.len() <= max_width {
        return dir.to_string();
    }

    let separator = detect_separator(dir);
    let parts: Vec<&str> = dir.split(separator).collect();

    if parts.len() <= 2 {
        let ellipsis = "...";
        let available = max_width.saturating_sub(ellipsis.len());
        let start_pos = dir.len().saturating_sub(available);
        return format!("{}{}", ellipsis, &dir[start_pos..]);
    }

    let (first_part, last_part) = (parts[0], parts[parts.len() - 1]);
    let ellipsis = "...";
    let base_length = first_part.len() + last_part.len() + ellipsis.len() + 2; // +2 for separators

    if base_length <= max_width {
        return format!("{first_part}{separator}{ellipsis}{separator}{last_part}");
    }

    // Truncate the last part if still too long
    let available_for_last = max_width.saturating_sub(first_part.len() + ellipsis.len() + 2);
    let truncated_last = if last_part.len() > available_for_last {
        &last_part[last_part.len().saturating_sub(available_for_last)..]
    } else {
        last_part
    };

    format!("{first_part}{separator}{ellipsis}{separator}{truncated_last}")
}

/// Truncate a directory path using middle ellipsis with separators.
///
/// This function truncates long directory paths by placing "\..." in the middle,
/// preserving the beginning and end of the path with proper separator handling.
///
/// # Arguments
///
/// * `dir` - The directory path to truncate
/// * `max_width` - Maximum width for the result string
///
/// # Returns
///
/// A truncated path string that fits within `max_width` characters
///
/// # Examples
///
/// ```
/// use nettoolskit_string_utils::string::truncate_directory_with_middle;
/// let truncated = truncate_directory_with_middle("C:\\very\\long\\path\\to\\project", 25);
/// // Returns something like "C:\\very\\...\\project"
/// ```
/// A string with middle parts replaced by "..." if truncation was needed
#[must_use]
pub fn truncate_directory_with_middle(dir: &str, max_width: usize) -> String {
    if dir.len() <= max_width {
        return dir.to_string();
    }

    let separator = detect_separator(dir);
    let ellipsis = format!("{separator}...");

    // Fallback for very short limits
    if max_width <= ellipsis.len() + 4 {
        let start_pos = dir.len().saturating_sub(max_width);
        return format!("...{}", &dir[start_pos..]);
    }

    // Filter out empty parts (except root)
    let parts: Vec<&str> = dir
        .split(separator)
        .enumerate()
        .filter(|(i, part)| *i == 0 || !part.is_empty())
        .map(|(_, part)| part)
        .collect();

    if parts.len() <= 2 {
        return simple_truncate(dir, max_width, &ellipsis, separator);
    }

    let available_space = max_width - ellipsis.len();
    let front_space = available_space * 35 / 100;
    let back_space = available_space - front_space;

    // Build front part with complete path components
    let mut front_str = String::new();
    let mut used_front = 0;

    for (i, part) in parts.iter().enumerate().take(parts.len() - 1) {
        let component = if i == 0 && part.is_empty() {
            separator.to_string() // Unix root "/"
        } else {
            format!("{part}{separator}")
        };

        if used_front + component.len() <= front_space {
            front_str.push_str(&component);
            used_front += component.len();
        } else {
            break;
        }
    }

    // Build back part from the end
    let mut back_parts = Vec::new();
    let mut used_back = 0;

    for i in (0..parts.len()).rev() {
        let part = parts[i];
        if part.is_empty() {
            continue;
        }

        let component_len = if back_parts.is_empty() {
            part.len() // Last component (no separator)
        } else {
            part.len() + 1 // Additional components need separator
        };

        if used_back + component_len <= back_space {
            back_parts.insert(0, part);
            used_back += component_len;
        } else {
            break;
        }
    }

    let back_str = back_parts.join(&separator.to_string());

    // Remove trailing separator to avoid duplication
    let clean_front = front_str.trim_end_matches(separator);

    // Construct result
    let result = if back_str.is_empty() {
        format!("{clean_front}{ellipsis}")
    } else {
        format!("{clean_front}{ellipsis}{separator}{back_str}")
    };

    // Fallback if still too long
    if result.len() > max_width {
        simple_truncate(dir, max_width, &ellipsis, separator)
    } else {
        result
    }
}
