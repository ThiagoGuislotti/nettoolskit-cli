use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, execute, queue};
use once_cell::sync::Lazy;
use std::cmp::min;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Clear the terminal screen and move cursor to top-left position.
///
/// This function performs a complete terminal reset, useful for
/// starting with a clean display state.
///
/// # Returns
///
/// Returns `Ok(())` if the terminal is cleared successfully,
/// or an `io::Error` if terminal operations fail.
///
/// # Examples
///
/// ```
/// use nettoolskit_ui::clear_terminal;
/// clear_terminal().expect("Failed to clear terminal");
/// ```
pub fn clear_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Explicit escape sequences for wide terminal compatibility
    //  - ESC[3J clears scrollback (where supported)
    //  - ESC[2J clears the visible screen
    //  - ESC[H moves cursor to home
    stdout.write_all(b"\x1b[3J\x1b[2J\x1b[H")?;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    io::stdout().flush()
}

const MIN_DYNAMIC_HEIGHT: u16 = 6;
const FOOTER_TARGET_HEIGHT: u16 = 10;

static ACTIVE_LAYOUT: Lazy<Mutex<Option<Arc<TerminalLayoutInner>>>> =
    Lazy::new(|| Mutex::new(None));
static PENDING_LOGS: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
static INTERACTIVE_MODE: AtomicBool = AtomicBool::new(false);

const PENDING_LOG_CAPACITY: usize = 256;

/// Guard that enables interactive logging mode while in scope.
pub struct InteractiveLogGuard {
    active: bool,
}

impl InteractiveLogGuard {
    /// Disable interactive logging immediately, keeping guard alive for drop ordering.
    pub fn deactivate(&mut self) {
        if self.active {
            disable_interactive_logging();
            self.active = false;
        }
    }
}

impl Drop for InteractiveLogGuard {
    fn drop(&mut self) {
        if self.active {
            disable_interactive_logging();
        }
    }
}

/// Enable interactive logging queueing, returning a guard that disables on drop.
pub fn begin_interactive_logging() -> InteractiveLogGuard {
    {
        let mut pending = PENDING_LOGS.lock().unwrap();
        pending.clear();
    }
    INTERACTIVE_MODE.store(true, Ordering::SeqCst);
    InteractiveLogGuard { active: true }
}

/// Disable interactive logging mode and clear pending buffers.
pub fn disable_interactive_logging() {
    INTERACTIVE_MODE.store(false, Ordering::SeqCst);
    let drained: Vec<String> = {
        let mut pending = PENDING_LOGS.lock().unwrap();
        pending.drain(..).collect()
    };

    if drained.is_empty() {
        return;
    }

    let mut stdout = io::stdout();
    for entry in drained {
        let _ = writeln!(stdout, "{entry}");
    }
    let _ = stdout.flush();
}

/// Manage the interactive terminal layout with fixed header and footer.
///
/// The layout reserves the top portion of the screen for the static header,
/// the bottom for log output, and constrains scrolling to the middle region.
pub struct TerminalLayout {
    inner: Arc<TerminalLayoutInner>,
}

struct TerminalLayoutInner {
    state: Mutex<LayoutState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutMetrics {
    width: u16,
    height: u16,
    footer_height: u16,
    footer_start: u16,
    scroll_top: u16,
    scroll_bottom: u16,
    log_capacity: usize,
}

fn calculate_layout_metrics(width: u16, height: u16) -> io::Result<LayoutMetrics> {
    let mut footer_height = FOOTER_TARGET_HEIGHT;

    if height < MIN_DYNAMIC_HEIGHT + footer_height {
        let available_footer = height.saturating_sub(MIN_DYNAMIC_HEIGHT);
        footer_height = available_footer.max(3);
    }

    let dynamic_height = height.saturating_sub(footer_height);
    if dynamic_height < MIN_DYNAMIC_HEIGHT {
        return Err(io::Error::other("Terminal height insufficient for layout"));
    }

    let footer_start = height.saturating_sub(footer_height);
    let scroll_top = 0;
    let scroll_bottom = footer_start.saturating_sub(1);
    let log_capacity = footer_height.saturating_sub(2).max(1) as usize;

    Ok(LayoutMetrics {
        width,
        height,
        footer_height,
        footer_start,
        scroll_top,
        scroll_bottom,
        log_capacity,
    })
}

struct LayoutState {
    metrics: LayoutMetrics,
    logs: VecDeque<String>,
}

impl TerminalLayout {
    /// Initialize terminal layout, render header/footer, and set scroll region.
    ///
    /// # Parameters
    ///
    /// * `render_header` - Optional function to render the header content (e.g., logo, welcome message)
    pub fn initialize<F>(render_header: Option<F>) -> io::Result<Self>
    where
        F: FnOnce(),
    {
        clear_terminal()?;
        if let Some(render) = render_header {
            render();
        }
        io::stdout().flush()?;

        let (width, height) = terminal::size()?;
        let metrics = calculate_layout_metrics(width, height)?;

        let inner = Arc::new(TerminalLayoutInner {
            state: Mutex::new(LayoutState {
                metrics,
                logs: VecDeque::with_capacity(metrics.log_capacity),
            }),
        });

        inner.render_footer()?;
        TerminalLayoutInner::activate(&inner)?;

        Ok(Self { inner })
    }

    /// Append a log line to the footer region.
    pub fn append_log_line(line: &str) -> io::Result<()> {
        append_footer_log(line)
    }
}

impl Drop for TerminalLayout {
    fn drop(&mut self) {
        if let Err(error) = self.inner.reset_scroll_region() {
            let _ = io::stderr()
                .write_all(format!("Failed to reset terminal layout: {error}\n").as_bytes());
        }
        let mut slot = ACTIVE_LAYOUT.lock().unwrap();
        *slot = None;
    }
}

impl TerminalLayoutInner {
    fn activate(inner: &Arc<Self>) -> io::Result<()> {
        {
            let mut slot = ACTIVE_LAYOUT.lock().unwrap();
            *slot = Some(inner.clone());
        }
        inner.flush_pending_logs()
    }

    fn reset_scroll_region(&self) -> io::Result<()> {
        // Move cursor to bottom before resetting scroll region
        // This prevents cursor from jumping to top when scroll region is reset
        let (_, height) = terminal::size()?;
        let mut stdout = io::stdout();

        // Move cursor to last line of terminal
        execute!(stdout, cursor::MoveTo(0, height.saturating_sub(1)))?;

        // Now reset scroll region
        reset_scroll_region_full()?;

        // Ensure cursor stays at bottom
        execute!(stdout, cursor::MoveTo(0, height.saturating_sub(1)))?;
        stdout.flush()
    }

    fn ensure_scroll_region(&self) -> io::Result<()> {
        let state = self.state.lock().unwrap();
        apply_scroll_region(state.metrics.scroll_top, state.metrics.scroll_bottom)
    }

    fn reconfigure(&self, metrics: LayoutMetrics) -> io::Result<()> {
        {
            let mut state = self.state.lock().unwrap();
            state.metrics = metrics;
            while state.logs.len() > state.metrics.log_capacity {
                state.logs.pop_front();
            }
        }

        clear_terminal()?;
        // Note: Header re-rendering on reconfigure is handled by the caller if needed
        self.render_footer()
    }

    fn render_footer(&self) -> io::Result<()> {
        let state = self.state.lock().unwrap();
        self.render_footer_locked(&state)
    }

    fn flush_pending_logs(&self) -> io::Result<()> {
        let mut pending = PENDING_LOGS.lock().unwrap();
        while let Some(entry) = pending.pop_front() {
            self.append_log_line(&entry)?;
        }
        Ok(())
    }

    fn render_footer_locked(&self, state: &LayoutState) -> io::Result<()> {
        let mut stdout = io::stdout();
        let metrics = state.metrics;
        let origin = cursor::position().unwrap_or((0, metrics.scroll_top));

        reset_scroll_region_full()?;
        execute!(
            stdout,
            cursor::MoveTo(0, metrics.footer_start),
            Clear(ClearType::FromCursorDown)
        )?;

        let separator = "-".repeat(metrics.width as usize);
        queue!(stdout, Print(&separator))?;

        for idx in 0..metrics.log_capacity {
            let content = state
                .logs
                .get(idx)
                .map_or(String::new(), |entry| pad_to_width(entry, metrics.width));
            queue!(
                stdout,
                cursor::MoveTo(0, metrics.footer_start + 1 + idx as u16),
                Print(content)
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(
                0,
                metrics.footer_start + metrics.footer_height.saturating_sub(1)
            ),
            Print(&separator)
        )?;

        apply_scroll_region(metrics.scroll_top, metrics.scroll_bottom)?;
        execute!(stdout, cursor::MoveTo(origin.0, origin.1))?;
        stdout.flush()
    }

    fn append_log_line(&self, line: &str) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        if state.metrics.log_capacity == 0 {
            return Ok(());
        }

        let Some(cleaned) = normalize_log_entry(line) else {
            return Ok(());
        };

        if state.logs.len() == state.metrics.log_capacity {
            state.logs.pop_front();
        }

        let truncated = truncate_to_width(&cleaned, state.metrics.width);
        state.logs.push_back(truncated);
        self.render_footer_locked(&state)
    }
}

fn normalize_log_entry(line: &str) -> Option<String> {
    let cleaned = line
        .trim_end_matches(&['\n', '\r'][..])
        .replace('\t', "    ");

    if cleaned.trim().is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn append_log_to_active_layout(line: &str) -> io::Result<bool> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap();
        slot.clone()
    };

    if let Some(active) = layout {
        active.append_log_line(line)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn pad_to_width(text: &str, width: u16) -> String {
    let width = width as usize;
    if width == 0 {
        return String::new();
    }

    let truncated = truncate_to_width(text, width as u16);
    let padding = width.saturating_sub(truncated.len());
    format!("{truncated}{:padding$}", "")
}

fn truncate_to_width(text: &str, width: u16) -> String {
    let max_len = width as usize;
    if text.len() <= max_len {
        return text.to_string();
    }

    let mut result = String::with_capacity(min(text.len(), max_len));
    for ch in text.chars() {
        if result.len() + ch.len_utf8() > max_len {
            break;
        }
        result.push(ch);
    }
    result
}

fn apply_scroll_region(top: u16, bottom: u16) -> io::Result<()> {
    if bottom < top {
        return Ok(());
    }

    let mut stdout = io::stdout();
    // Save cursor position before applying scroll region
    let cursor_pos = cursor::position().unwrap_or((0, top));

    write!(stdout, "\x1b[{};{}r", top + 1, bottom + 1)?;

    // Restore cursor position after applying scroll region
    execute!(stdout, cursor::MoveTo(cursor_pos.0, cursor_pos.1))?;
    stdout.flush()
}

fn reset_scroll_region_full() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b[r")?;
    stdout.flush()
}

pub fn ensure_layout_integrity() -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap();
        slot.clone()
    };

    if let Some(active) = layout {
        active.ensure_scroll_region()
    } else {
        Ok(())
    }
}

pub fn handle_resize(width: u16, height: u16) -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap();
        slot.clone()
    };

    if let Some(active) = layout {
        let metrics = calculate_layout_metrics(width, height)?;
        active.reconfigure(metrics)?;
    }

    Ok(())
}

/// Append a formatted log line to the footer; fallback to stdout if layout is inactive.
pub fn append_footer_log(line: &str) -> io::Result<()> {
    let Some(entry) = normalize_log_entry(line) else {
        return Ok(());
    };

    if append_log_to_active_layout(&entry)? {
        return Ok(());
    }

    if INTERACTIVE_MODE.load(Ordering::SeqCst) {
        let mut pending = PENDING_LOGS.lock().unwrap();
        if pending.len() == PENDING_LOG_CAPACITY {
            pending.pop_front();
        }
        pending.push_back(entry);
        Ok(())
    } else {
        let mut stdout = io::stdout();
        stdout.write_all(entry.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_metrics_respect_task02_contract() {
        let metrics = calculate_layout_metrics(120, 40).expect("metrics");
        assert_eq!(metrics.scroll_top, 0);
        assert_eq!(metrics.footer_start + metrics.footer_height, metrics.height);
        assert_eq!(metrics.scroll_bottom + 1, metrics.footer_start);
        assert!(metrics.footer_start >= MIN_DYNAMIC_HEIGHT);
        assert!(metrics.log_capacity >= 1);
    }

    #[test]
    fn layout_metrics_fail_when_terminal_too_small() {
        assert!(calculate_layout_metrics(80, MIN_DYNAMIC_HEIGHT - 1).is_err());
    }
}
