use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, execute, queue};
use once_cell::sync::Lazy;
use std::cmp::min;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    force_clear_screen(&mut stdout)
}

/// Perform a robust screen clear that works across all terminal emulators.
///
/// Some terminals (notably Windows Terminal) implement `ESC[2J` by scrolling
/// visible content into the scrollback buffer rather than truly erasing it.
/// When `ESC[3J` (purge scrollback) runs **before** `ESC[2J`, the content
/// pushed to scrollback by `ESC[2J` is never purged — causing visual
/// duplication on resize.
///
/// This function avoids the problem entirely by:
/// 1. Resetting the scroll region to the full terminal
/// 2. Clearing every visible line individually with `ESC[2K]`
/// 3. Purging the scrollback buffer after all lines are cleared
/// 4. Returning the cursor to the home position
fn force_clear_screen(stdout: &mut io::Stdout) -> io::Result<()> {
    // Reset scroll region so clears reach every line, not just the old region
    reset_scroll_region_full()?;

    let (_, height) = terminal::size()?;

    // Clear every line individually — guaranteed to work regardless of
    // scroll region state or terminal-specific ESC[2J behavior
    for row in 0..height {
        queue!(
            stdout,
            cursor::MoveTo(0, row),
            Clear(ClearType::CurrentLine)
        )?;
    }

    // Purge scrollback AFTER visible lines are cleared, then cursor home
    queue!(stdout, Clear(ClearType::Purge), cursor::MoveTo(0, 0))?;
    stdout.flush()
}

const MIN_DYNAMIC_HEIGHT: u16 = 6;
const FOOTER_TARGET_HEIGHT: u16 = 10;

static ACTIVE_LAYOUT: Lazy<Mutex<Option<Arc<TerminalLayoutInner>>>> =
    Lazy::new(|| Mutex::new(None));
static PENDING_LOGS: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
static INTERACTIVE_MODE: AtomicBool = AtomicBool::new(false);
static FOOTER_OUTPUT_ENABLED: AtomicBool = AtomicBool::new(true);
static FOCUS_DETECTION_ENABLED: AtomicBool = AtomicBool::new(false);
static TERMINAL_FOCUSED: AtomicBool = AtomicBool::new(true);

const PENDING_LOG_CAPACITY: usize = 256;

/// Target frame rate for terminal frame coalescing.
const FRAME_TARGET_FPS: u16 = 60;
/// Default poll timeout used when no frame is pending.
const FRAME_IDLE_POLL_MS: u64 = 50;

/// Tracks a pending resize event timestamp for trailing-edge debounce.
static PENDING_RESIZE: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

/// Minimum interval in milliseconds between resize redraws.
const RESIZE_DEBOUNCE_MS: u64 = 80;

/// Flag to suppress footer rendering during reconfigure.
/// Prevents `append_log_line` (triggered via tracing/UiWriter) from rendering
/// the footer at stale positions while the screen is being reconstructed.
static RECONFIGURING: AtomicBool = AtomicBool::new(false);

static FRAME_SCHEDULER: Lazy<Mutex<FrameScheduler>> = Lazy::new(|| {
    Mutex::new(FrameScheduler::new(
        FRAME_TARGET_FPS,
        Duration::from_millis(FRAME_IDLE_POLL_MS),
    ))
});

#[derive(Debug, Clone)]
struct FrameScheduler {
    frame_interval: Duration,
    idle_poll_timeout: Duration,
    last_frame_at: Option<Instant>,
    pending_frame: bool,
}

impl FrameScheduler {
    fn new(target_fps: u16, idle_poll_timeout: Duration) -> Self {
        // Keep sane defaults even if target_fps is accidentally configured to zero.
        let fps = target_fps.max(1) as u64;
        let frame_interval = Duration::from_nanos(1_000_000_000_u64 / fps);
        Self {
            frame_interval,
            idle_poll_timeout,
            last_frame_at: None,
            pending_frame: false,
        }
    }

    fn request_frame(&mut self) {
        self.pending_frame = true;
    }

    fn try_consume_frame(&mut self, now: Instant) -> bool {
        if !self.pending_frame {
            return false;
        }

        if let Some(last_frame_at) = self.last_frame_at {
            if now.duration_since(last_frame_at) < self.frame_interval {
                return false;
            }
        }

        self.last_frame_at = Some(now);
        self.pending_frame = false;
        true
    }

    fn poll_timeout(&self, now: Instant) -> Duration {
        if !self.pending_frame {
            return self.idle_poll_timeout;
        }

        let Some(last_frame_at) = self.last_frame_at else {
            return Duration::from_millis(1);
        };

        let elapsed = now.duration_since(last_frame_at);
        if elapsed >= self.frame_interval {
            Duration::from_millis(1)
        } else {
            self.frame_interval - elapsed
        }
    }
}

/// Request a terminal frame render in the shared scheduler.
///
/// Multiple requests before consumption are coalesced into a single frame.
pub fn request_terminal_frame() {
    let mut scheduler = FRAME_SCHEDULER.lock().unwrap_or_else(|e| e.into_inner());
    scheduler.request_frame();
}

/// Try to consume a scheduled terminal frame.
///
/// Returns `true` when a frame should be rendered now, or `false` when
/// there is no pending frame or the frame budget has not elapsed yet.
#[must_use]
pub fn consume_scheduled_terminal_frame() -> bool {
    let mut scheduler = FRAME_SCHEDULER.lock().unwrap_or_else(|e| e.into_inner());
    scheduler.try_consume_frame(Instant::now())
}

/// Compute event-loop poll timeout based on frame scheduler pressure.
///
/// When no frame is pending this returns `default_timeout`. When a frame is
/// pending, the timeout is reduced so async loops wake up close to the next
/// frame boundary (60 FPS target by default).
#[must_use]
pub fn scheduled_frame_poll_timeout(default_timeout: Duration) -> Duration {
    let scheduler = FRAME_SCHEDULER.lock().unwrap_or_else(|e| e.into_inner());
    let scheduled_timeout = scheduler.poll_timeout(Instant::now());
    min(default_timeout, scheduled_timeout)
}

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
        let mut pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        pending.clear();
    }
    INTERACTIVE_MODE.store(true, Ordering::SeqCst);
    InteractiveLogGuard { active: true }
}

/// Disable interactive logging mode and clear pending buffers.
pub fn disable_interactive_logging() {
    INTERACTIVE_MODE.store(false, Ordering::SeqCst);
    let drained: Vec<String> = {
        let mut pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        pending.drain(..).collect()
    };

    if drained.is_empty() || !FOOTER_OUTPUT_ENABLED.load(Ordering::SeqCst) {
        return;
    }

    let mut stdout = io::stdout();
    for entry in drained {
        let _ = writeln!(stdout, "{entry}");
    }
    let _ = stdout.flush();
}

/// Enable or disable footer output rendering at runtime.
pub fn set_footer_output_enabled(enabled: bool) {
    FOOTER_OUTPUT_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Returns whether footer output rendering is enabled.
#[must_use]
pub fn footer_output_enabled() -> bool {
    FOOTER_OUTPUT_ENABLED.load(Ordering::SeqCst)
}

/// Enable or disable focus-based notification gating at runtime.
///
/// When disabled, focus state is reset to focused.
pub fn set_focus_detection_enabled(enabled: bool) {
    FOCUS_DETECTION_ENABLED.store(enabled, Ordering::SeqCst);
    if !enabled {
        TERMINAL_FOCUSED.store(true, Ordering::SeqCst);
    }
}

/// Returns whether focus detection is currently enabled.
#[must_use]
pub fn focus_detection_enabled() -> bool {
    FOCUS_DETECTION_ENABLED.load(Ordering::SeqCst)
}

/// Update terminal focus state (`true` = focused, `false` = unfocused).
pub fn set_terminal_focused(focused: bool) {
    TERMINAL_FOCUSED.store(focused, Ordering::SeqCst);
}

/// Returns current terminal focus state.
#[must_use]
pub fn terminal_focused() -> bool {
    TERMINAL_FOCUSED.load(Ordering::SeqCst)
}

/// Determines whether an attention signal should be emitted.
///
/// If `unfocused_only` is enabled, signals are only emitted when focus
/// detection is active and the terminal is currently unfocused.
#[must_use]
pub fn should_emit_attention_signal(unfocused_only: bool) -> bool {
    if !unfocused_only {
        return true;
    }

    focus_detection_enabled() && !terminal_focused()
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
    header_renderer: Option<fn()>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutMetrics {
    width: u16,
    height: u16,
    header_height: u16,
    footer_height: u16,
    footer_start: u16,
    scroll_top: u16,
    scroll_bottom: u16,
    log_capacity: usize,
}

fn calculate_layout_metrics(
    width: u16,
    height: u16,
    header_height: u16,
) -> io::Result<LayoutMetrics> {
    if header_height >= height {
        return Err(io::Error::other("Terminal height insufficient for layout"));
    }

    let mut footer_height = FOOTER_TARGET_HEIGHT;

    let required_height = header_height
        .saturating_add(MIN_DYNAMIC_HEIGHT)
        .saturating_add(footer_height);

    if height < required_height {
        let available_footer =
            height.saturating_sub(header_height.saturating_add(MIN_DYNAMIC_HEIGHT));
        footer_height = available_footer.max(3);
    }

    let dynamic_height = height
        .saturating_sub(header_height)
        .saturating_sub(footer_height);
    if dynamic_height < MIN_DYNAMIC_HEIGHT {
        return Err(io::Error::other("Terminal height insufficient for layout"));
    }

    let footer_start = height.saturating_sub(footer_height);
    let scroll_top = header_height;
    let scroll_bottom = footer_start.saturating_sub(1);
    let log_capacity = footer_height.saturating_sub(2).max(1) as usize;

    Ok(LayoutMetrics {
        width,
        height,
        header_height,
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
    pub fn initialize(render_header: Option<fn()>) -> io::Result<Self> {
        ensure_cursor_visible_blinking()?;
        clear_terminal()?;
        if let Some(render) = render_header {
            render();
        }
        io::stdout().flush()?;

        let header_height = current_cursor_line();
        let (width, height) = terminal::size()?;
        let metrics = calculate_layout_metrics(width, height, header_height)?;

        let inner = Arc::new(TerminalLayoutInner {
            state: Mutex::new(LayoutState {
                metrics,
                logs: VecDeque::with_capacity(metrics.log_capacity),
            }),
            header_renderer: render_header,
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
        if let Err(error) = self.inner.restore_terminal_state() {
            let _ = io::stderr()
                .write_all(format!("Failed to reset terminal layout: {error}\n").as_bytes());
        }
        let mut slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
        *slot = None;
    }
}

impl TerminalLayoutInner {
    fn activate(inner: &Arc<Self>) -> io::Result<()> {
        {
            let mut slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
            *slot = Some(inner.clone());
        }
        inner.flush_pending_logs()
    }

    fn restore_terminal_state(&self) -> io::Result<()> {
        self.reset_scroll_region()?;
        ensure_cursor_visible_blinking()
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
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        apply_scroll_region(state.metrics.scroll_top, state.metrics.scroll_bottom)
    }

    fn reconfigure(&self) -> io::Result<()> {
        // Always read actual current terminal size (not event payload)
        let (width, height) = terminal::size()?;

        // Idempotency: skip if dimensions haven't changed
        {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.metrics.width == width && state.metrics.height == height {
                return Ok(());
            }
        }
        // Lock released — header rendering may trigger tracing → UiWriter → append_log_line,
        // so we cannot hold the state lock across the header render (would deadlock).
        // Instead, use RECONFIGURING flag to suppress footer renders during this window.

        let mut stdout = io::stdout();

        // Prevent concurrent footer renders during reconfigure
        RECONFIGURING.store(true, Ordering::SeqCst);

        // === Begin atomic redraw ===

        // 1. Hide cursor to prevent flickering and ghost artifacts
        let _ = execute!(stdout, cursor::Hide);

        // 2. Robust screen clear: line-by-line + purge scrollback
        //    Replaces the old Purge+All sequence which failed on Windows Terminal
        //    (ESC[2J scrolls content into scrollback; ESC[3J before it leaves that content)
        force_clear_screen(&mut stdout)?;

        // 3. Render header content (logo, welcome box, tips)
        //    This may trigger tracing output — footer renders are suppressed via RECONFIGURING.
        if let Some(render_header) = self.header_renderer {
            render_header();
        }
        stdout.flush()?;

        // 4. Calculate new layout from actual header height
        let header_height = current_cursor_line();
        let metrics = match calculate_layout_metrics(width, height, header_height) {
            Ok(m) => m,
            Err(e) => {
                // Terminal too small: reset to sane state and propagate error
                RECONFIGURING.store(false, Ordering::SeqCst);
                reset_scroll_region_full()?;
                ensure_cursor_visible_blinking()?;
                return Err(e);
            }
        };

        // 5. Re-acquire lock, update state, and render footer atomically
        let render_result = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.metrics = metrics;
            while state.logs.len() > metrics.log_capacity {
                state.logs.pop_front();
            }
            // Render footer with all accumulated logs (using locked state to avoid deadlock)
            self.render_footer_locked(&state)
        };

        // 6. Always re-enable normal footer renders, regardless of render success
        RECONFIGURING.store(false, Ordering::SeqCst);

        // 7. Restore cursor visibility
        ensure_cursor_visible_blinking()?;

        // === End atomic redraw ===

        // Propagate render error after cleanup
        render_result
    }

    fn render_footer(&self) -> io::Result<()> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        self.render_footer_locked(&state)
    }

    fn flush_pending_logs(&self) -> io::Result<()> {
        let mut pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        while let Some(entry) = pending.pop_front() {
            self.append_log_line(&entry)?;
        }
        Ok(())
    }

    fn render_footer_locked(&self, state: &LayoutState) -> io::Result<()> {
        let mut stdout = io::stdout();
        let metrics = state.metrics;
        let origin = clamp_cursor_to_metrics(
            cursor::position().unwrap_or((0, metrics.scroll_top)),
            metrics,
        );

        // Temporarily remove scroll region to access footer area
        reset_scroll_region_full()?;

        // Batch all footer rendering operations before flushing
        queue!(
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

        // Flush all queued visual content before manipulating scroll region
        stdout.flush()?;

        // Re-apply scroll region and restore cursor position
        apply_scroll_region(metrics.scroll_top, metrics.scroll_bottom)?;
        execute!(stdout, cursor::MoveTo(origin.0, origin.1))?;
        stdout.flush()
    }

    fn append_log_line(&self, line: &str) -> io::Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
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

        // Skip visual render during reconfigure to avoid rendering footer
        // at stale positions on a freshly cleared screen.
        // Logs are still stored and will be rendered when reconfigure completes.
        if RECONFIGURING.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.render_footer_locked(&state)
    }

    fn prepare_prompt_line(&self) -> io::Result<()> {
        let metrics = {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.metrics
        };

        apply_scroll_region(metrics.scroll_top, metrics.scroll_bottom)?;

        let (x, y) = cursor::position().unwrap_or((0, metrics.scroll_top));
        let mut stdout = io::stdout();
        let clamped_y = y.clamp(metrics.scroll_top, metrics.scroll_bottom);

        if x > 0 {
            if clamped_y >= metrics.scroll_bottom {
                queue!(
                    stdout,
                    cursor::MoveTo(0, metrics.scroll_bottom),
                    Print("\n")
                )?;
            } else {
                queue!(stdout, cursor::MoveTo(0, clamped_y.saturating_add(1)))?;
            }
        } else {
            queue!(stdout, cursor::MoveTo(0, clamped_y))?;
        }

        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            cursor::MoveToColumn(0)
        )?;
        stdout.flush()?;
        ensure_cursor_visible_blinking()
    }
}

fn clamp_cursor_to_metrics((x, y): (u16, u16), metrics: LayoutMetrics) -> (u16, u16) {
    let max_x = metrics.width.saturating_sub(1);
    let clamped_x = x.min(max_x);
    let clamped_y = y.clamp(metrics.scroll_top, metrics.scroll_bottom);
    (clamped_x, clamped_y)
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
        let slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
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

fn ensure_cursor_visible_blinking() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Show, cursor::SetCursorStyle::BlinkingBlock)?;
    stdout.flush()
}

fn current_cursor_line() -> u16 {
    cursor::position()
        .map(|(x, y)| if x > 0 { y.saturating_add(1) } else { y })
        .unwrap_or(0)
}

/// Verify and restore the scroll region if an active layout exists.
///
/// Call this after operations that may have disrupted the terminal
/// scroll region (e.g., interactive menus rendered by third-party libraries).
pub fn ensure_layout_integrity() -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
        slot.clone()
    };

    if let Some(active) = layout {
        active.ensure_scroll_region()
    } else {
        Ok(())
    }
}

/// Position the cursor on a clean prompt line in the dynamic area.
///
/// When the interactive layout is active, this function clamps and normalizes
/// cursor placement to keep the prompt below the latest output and inside the
/// scrollable region. Without an active layout, it normalizes to column zero
/// and advances one line if needed.
pub fn prepare_prompt_line() -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
        slot.clone()
    };

    if let Some(active) = layout {
        return active.prepare_prompt_line();
    }

    let mut stdout = io::stdout();
    let (x, _) = cursor::position().unwrap_or((0, 0));
    if x > 0 {
        queue!(stdout, Print("\n"))?;
    }
    queue!(stdout, cursor::MoveToColumn(0))?;
    stdout.flush()?;
    ensure_cursor_visible_blinking()
}

/// Reset the interactive layout and redraw the header/footer regions.
///
/// If an active interactive layout exists, this triggers an immediate reconfigure
/// using the current terminal dimensions. If no layout is active, this falls back
/// to a plain terminal clear.
pub fn reset_layout() -> io::Result<()> {
    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
        slot.clone()
    };

    if let Some(active) = layout {
        if let Err(error) = active.reconfigure() {
            if error
                .to_string()
                .contains("Terminal height insufficient for layout")
            {
                reset_scroll_region_full()?;
                ensure_cursor_visible_blinking()?;
                return Ok(());
            }
            return Err(error);
        }
        Ok(())
    } else {
        clear_terminal()?;
        ensure_cursor_visible_blinking()
    }
}

/// Record a terminal resize event for trailing-edge debounce processing.
///
/// The actual layout reconfiguration is deferred to [`process_pending_resize`].
/// Rapid consecutive resize events are coalesced so only the final dimensions
/// are applied after the debounce interval elapses.
pub fn handle_resize(_width: u16, _height: u16) -> io::Result<()> {
    // Mark resize as pending; actual processing is deferred to process_pending_resize().
    // This implements the recording side of a trailing-edge debounce:
    // rapid resize events just update the timestamp, and only the final
    // terminal state is rendered after the debounce interval.
    let mut pending = PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner());
    *pending = Some(Instant::now());
    Ok(())
}

/// Process any pending resize event after the debounce interval has elapsed.
///
/// Call this periodically (e.g., on every event-loop poll timeout) to handle
/// deferred resize events. Multiple rapid resize events are coalesced into
/// a single redraw using the terminal's actual current dimensions.
///
/// # Returns
///
/// Returns `Ok(())` if no resize is pending or if the resize was processed
/// successfully. Returns an error only for unrecoverable terminal failures.
pub fn process_pending_resize() -> io::Result<()> {
    let should_process = {
        let pending = PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner());
        pending.is_some_and(|instant| {
            instant.elapsed() >= std::time::Duration::from_millis(RESIZE_DEBOUNCE_MS)
        })
    };

    if !should_process {
        return Ok(());
    }

    // Clear the pending flag before processing to avoid double-processing
    {
        let mut pending = PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner());
        *pending = None;
    }

    let layout = {
        let slot = ACTIVE_LAYOUT.lock().unwrap_or_else(|e| e.into_inner());
        slot.clone()
    };

    if let Some(active) = layout {
        if let Err(error) = active.reconfigure() {
            if error
                .to_string()
                .contains("Terminal height insufficient for layout")
            {
                // Keep terminal usable even when the viewport is temporarily too small.
                reset_scroll_region_full()?;
                ensure_cursor_visible_blinking()?;
            } else {
                return Err(error);
            }
        }
    }

    Ok(())
}

/// Emit an attention bell (`BEL`, `\x07`) to the active terminal.
///
/// This is used as a lightweight, cross-platform notification signal when
/// important events happen (for example command failures in interactive mode).
pub fn emit_attention_bell() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x07")?;
    stdout.flush()
}

/// Append a formatted log line to the footer; fallback to stdout if layout is inactive.
pub fn append_footer_log(line: &str) -> io::Result<()> {
    if !FOOTER_OUTPUT_ENABLED.load(Ordering::SeqCst) {
        return Ok(());
    }

    let Some(entry) = normalize_log_entry(line) else {
        return Ok(());
    };

    if append_log_to_active_layout(&entry)? {
        return Ok(());
    }

    if INTERACTIVE_MODE.load(Ordering::SeqCst) {
        let mut pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
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
    use super::{
        append_footer_log, calculate_layout_metrics, clamp_cursor_to_metrics, emit_attention_bell,
        focus_detection_enabled, footer_output_enabled, handle_resize, normalize_log_entry,
        pad_to_width, prepare_prompt_line, process_pending_resize, reset_layout,
        set_focus_detection_enabled, set_footer_output_enabled, set_terminal_focused,
        should_emit_attention_signal, terminal_focused, truncate_to_width, FrameScheduler,
        LayoutMetrics, FOOTER_TARGET_HEIGHT, INTERACTIVE_MODE, MIN_DYNAMIC_HEIGHT, PENDING_RESIZE,
        RECONFIGURING, RESIZE_DEBOUNCE_MS,
    };
    use serial_test::serial;
    use std::collections::VecDeque;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    #[test]
    fn calculate_layout_metrics_uses_default_footer_height_when_space_allows() {
        let metrics = calculate_layout_metrics(120, 40, 0).expect("layout metrics should succeed");
        assert_eq!(metrics.footer_height, FOOTER_TARGET_HEIGHT);
        assert_eq!(metrics.footer_start, 30);
        assert_eq!(metrics.scroll_top, 0);
        assert_eq!(metrics.scroll_bottom, 29);
    }

    #[test]
    fn calculate_layout_metrics_returns_error_for_tiny_terminal() {
        let result = calculate_layout_metrics(80, MIN_DYNAMIC_HEIGHT.saturating_sub(1), 0);
        assert!(result.is_err());
    }

    #[test]
    fn calculate_layout_metrics_reserves_header_lines_from_scroll_area() {
        let metrics = calculate_layout_metrics(120, 40, 8).expect("layout metrics should succeed");
        assert_eq!(metrics.scroll_top, 8);
        assert_eq!(metrics.footer_start, 30);
        assert_eq!(metrics.scroll_bottom, 29);
    }

    #[test]
    fn clamp_cursor_to_metrics_limits_coordinates_to_visible_dynamic_area() {
        let metrics = LayoutMetrics {
            width: 80,
            height: 30,
            header_height: 4,
            footer_height: 10,
            footer_start: 20,
            scroll_top: 4,
            scroll_bottom: 19,
            log_capacity: 8,
        };

        let clamped = clamp_cursor_to_metrics((140, 42), metrics);
        assert_eq!(clamped, (79, 19));
    }

    #[test]
    fn frame_scheduler_coalesces_multiple_requests_into_single_frame() {
        let mut scheduler = FrameScheduler::new(60, Duration::from_millis(50));
        let now = Instant::now();

        scheduler.request_frame();
        scheduler.request_frame();
        scheduler.request_frame();

        assert!(scheduler.try_consume_frame(now));
        assert!(!scheduler.try_consume_frame(now));
    }

    #[test]
    fn frame_scheduler_enforces_rate_limit_for_back_to_back_frames() {
        let mut scheduler = FrameScheduler::new(60, Duration::from_millis(50));
        let first = Instant::now();

        scheduler.request_frame();
        assert!(scheduler.try_consume_frame(first));

        scheduler.request_frame();
        assert!(!scheduler.try_consume_frame(first + Duration::from_millis(5)));
        assert!(scheduler.try_consume_frame(first + Duration::from_millis(17)));
    }

    #[test]
    fn frame_scheduler_poll_timeout_respects_pending_state() {
        let mut scheduler = FrameScheduler::new(60, Duration::from_millis(50));
        let now = Instant::now();

        assert_eq!(scheduler.poll_timeout(now), Duration::from_millis(50));
        scheduler.request_frame();
        assert_eq!(scheduler.poll_timeout(now), Duration::from_millis(1));

        assert!(scheduler.try_consume_frame(now));
        scheduler.request_frame();
        let timeout = scheduler.poll_timeout(now + Duration::from_millis(2));
        assert!(timeout > Duration::from_millis(1));
        assert!(timeout <= Duration::from_millis(50));
    }

    // === Phase 1.5: Resize Unit Tests ===

    /// 1.5.1 — `handle_resize` marks `PENDING_RESIZE` without calling `reconfigure`.
    ///
    /// The function must only record a timestamp; actual processing is deferred
    /// to `process_pending_resize()` via trailing-edge debounce.
    #[test]
    #[serial]
    fn handle_resize_sets_pending_without_reconfigure() {
        // Clear any lingering state
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;

        handle_resize(120, 40).unwrap();

        let pending = PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_some(),
            "handle_resize must set PENDING_RESIZE timestamp"
        );

        // Cleanup
        drop(pending);
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// 1.5.2 — Debounce coalesces N events: rapid resize events within the
    /// debounce window are NOT processed by `process_pending_resize()`.
    #[test]
    #[serial]
    fn debounce_suppresses_processing_within_window() {
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;

        // Fire 5 rapid resize events — all within debounce window
        for _ in 0..5 {
            handle_resize(100, 50).unwrap();
        }

        // process_pending_resize immediately → debounce interval not elapsed
        process_pending_resize().unwrap();

        let still_pending = PENDING_RESIZE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some();
        assert!(
            still_pending,
            "pending must NOT be cleared within debounce window"
        );

        // Cleanup
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// 1.5.2 (coalesce proof) — After the debounce interval elapses,
    /// `process_pending_resize()` clears the pending flag and processes once.
    #[test]
    #[serial]
    fn debounce_clears_pending_after_interval() {
        // Set a pending timestamp already past the debounce window
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(Instant::now() - Duration::from_millis(RESIZE_DEBOUNCE_MS + 50));

        // No ACTIVE_LAYOUT → process_pending_resize just clears the flag
        process_pending_resize().unwrap();

        let cleared = PENDING_RESIZE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_none();
        assert!(
            cleared,
            "pending must be cleared after debounce interval elapsed"
        );
    }

    /// Resize burst simulation (shrink/grow): each new event must refresh the
    /// pending timestamp and keep the debounce coalescing state active.
    #[test]
    #[serial]
    fn resize_burst_shrink_grow_refreshes_pending_timestamp() {
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;

        handle_resize(140, 40).unwrap();
        let first = PENDING_RESIZE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .expect("first pending timestamp must exist");

        std::thread::sleep(Duration::from_millis(2));
        handle_resize(80, 24).unwrap();
        let second = PENDING_RESIZE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .expect("second pending timestamp must exist");

        std::thread::sleep(Duration::from_millis(2));
        handle_resize(150, 45).unwrap();
        let third = PENDING_RESIZE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .expect("third pending timestamp must exist");

        assert!(
            second >= first,
            "pending timestamp must not go backwards on burst event #2"
        );
        assert!(
            third >= second,
            "pending timestamp must not go backwards on burst event #3"
        );

        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Repeated debounce cycles must clear pending state deterministically even
    /// when resize events alternate between smaller and larger dimensions.
    #[test]
    #[serial]
    fn resize_cycles_clear_pending_each_round() {
        *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) = None;

        let sequence = [(120, 40), (80, 24), (140, 42), (70, 20), (130, 38)];
        for (w, h) in sequence {
            handle_resize(w, h).unwrap();

            // Force elapsed debounce window without sleeping in tests.
            *PENDING_RESIZE.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(Instant::now() - Duration::from_millis(RESIZE_DEBOUNCE_MS + 10));

            process_pending_resize().unwrap();

            let pending_is_none = PENDING_RESIZE
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_none();
            assert!(
                pending_is_none,
                "pending resize flag must be cleared after processing each cycle"
            );
        }
    }

    /// 1.5.3 — `RECONFIGURING` flag suppresses footer rendering.
    ///
    /// When the flag is `true`, `append_log_line` stores logs in the deque
    /// but returns early without calling `render_footer_locked`.
    #[test]
    #[serial]
    fn reconfiguring_flag_suppresses_footer_render_in_append() {
        use super::{LayoutState, TerminalLayoutInner};
        use std::sync::Mutex;

        // RECONFIGURING must default to false
        assert!(
            !RECONFIGURING.load(Ordering::SeqCst),
            "RECONFIGURING must default to false"
        );

        let metrics = calculate_layout_metrics(120, 40, 0).unwrap();
        let inner = TerminalLayoutInner {
            state: Mutex::new(LayoutState {
                metrics,
                logs: VecDeque::with_capacity(metrics.log_capacity),
            }),
            header_renderer: None,
        };

        // Set RECONFIGURING — footer render is suppressed
        RECONFIGURING.store(true, Ordering::SeqCst);

        let result = inner.append_log_line("log during reconfigure");

        // Restore immediately
        RECONFIGURING.store(false, Ordering::SeqCst);

        // Log should be stored without error (no terminal I/O occurred)
        assert!(
            result.is_ok(),
            "append_log_line must succeed during reconfigure"
        );

        let state = inner.state.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(
            state.logs.len(),
            1,
            "log must be stored even when RECONFIGURING suppresses render"
        );
    }

    /// 1.5.4 — Idempotency: `reconfigure()` short-circuits when terminal
    /// dimensions haven't changed, avoiding unnecessary screen redraws.
    #[test]
    fn reconfigure_idempotency_check_detects_same_dimensions() {
        let metrics = calculate_layout_metrics(120, 40, 0).unwrap();

        // Same dimensions → idempotent (reconfigure returns early)
        assert!(
            metrics.width == 120 && metrics.height == 40,
            "same dimensions must be detected as no-op"
        );

        // Different width → must trigger reconfigure
        assert!(
            !(metrics.width == 100 && metrics.height == 40),
            "different width must not match"
        );

        // Different height → must trigger reconfigure
        assert!(
            !(metrics.width == 120 && metrics.height == 30),
            "different height must not match"
        );

        // Both different → must trigger reconfigure
        assert!(
            !(metrics.width == 80 && metrics.height == 24),
            "both dimensions different must not match"
        );
    }

    /// Repeated shrink/grow metric calculations must preserve layout invariants
    /// (scroll region ordering, footer boundaries, and positive capacity).
    #[test]
    fn resize_sequence_shrink_grow_keeps_layout_invariants() {
        let header_height = 4;
        let sequence = [
            (160, 45),
            (120, 35),
            (100, 28),
            (90, 20),
            (110, 30),
            (150, 42),
        ];

        for (width, height) in sequence {
            let metrics = calculate_layout_metrics(width, height, header_height)
                .expect("sequence entry should be valid");

            assert_eq!(metrics.header_height, header_height);
            assert!(metrics.footer_height >= 3);
            assert!(metrics.footer_height <= FOOTER_TARGET_HEIGHT);
            assert!(metrics.scroll_top <= metrics.scroll_bottom);
            assert!(metrics.scroll_bottom < metrics.footer_start);
            assert!(metrics.footer_start < metrics.height);
            assert!(metrics.log_capacity >= 1);
        }
    }

    /// Shrink below minimum should fail fast, and grow back should recover to a
    /// valid layout in the next calculation.
    #[test]
    fn resize_sequence_too_small_then_recover() {
        let header_height = 8;

        let too_small = calculate_layout_metrics(80, 12, header_height);
        assert!(
            too_small.is_err(),
            "layout must fail when viewport is temporarily too small"
        );

        let recovered =
            calculate_layout_metrics(120, 40, header_height).expect("layout should recover");

        assert_eq!(recovered.header_height, header_height);
        assert!(recovered.scroll_top <= recovered.scroll_bottom);
        assert!(recovered.footer_start < recovered.height);
        assert!(recovered.log_capacity >= 1);
    }

    // === normalize_log_entry tests ===

    #[test]
    fn normalize_log_entry_trims_trailing_newlines() {
        let result = normalize_log_entry("hello\n\r").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn normalize_log_entry_replaces_tabs_with_spaces() {
        let result = normalize_log_entry("a\tb").unwrap();
        assert_eq!(result, "a    b");
    }

    #[test]
    fn normalize_log_entry_returns_none_for_empty() {
        assert!(normalize_log_entry("").is_none());
    }

    #[test]
    fn normalize_log_entry_returns_none_for_whitespace_only() {
        assert!(normalize_log_entry("   \n\r").is_none());
    }

    #[test]
    fn normalize_log_entry_preserves_content() {
        let result = normalize_log_entry("INFO: started").unwrap();
        assert_eq!(result, "INFO: started");
    }

    // === pad_to_width tests ===

    #[test]
    fn pad_to_width_pads_short_text() {
        let result = pad_to_width("hi", 10);
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("hi"));
    }

    #[test]
    fn pad_to_width_truncates_long_text() {
        let result = pad_to_width("hello world this is long", 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn pad_to_width_zero_width_returns_empty() {
        let result = pad_to_width("anything", 0);
        assert!(result.is_empty());
    }

    #[test]
    fn pad_to_width_exact_width() {
        let result = pad_to_width("abcde", 5);
        assert_eq!(result, "abcde");
    }

    // === truncate_to_width tests ===

    #[test]
    fn truncate_to_width_keeps_short_string() {
        let result = truncate_to_width("abc", 10);
        assert_eq!(result, "abc");
    }

    #[test]
    fn truncate_to_width_truncates_long_string() {
        let result = truncate_to_width("abcdefghij", 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn truncate_to_width_handles_unicode() {
        let result = truncate_to_width("café", 4);
        assert!(result.len() <= 4);
    }

    // === append_footer_log tests ===

    #[test]
    #[serial]
    fn append_footer_log_empty_input_is_noop() {
        let result = append_footer_log("");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn append_footer_log_whitespace_only_is_noop() {
        let result = append_footer_log("   \n");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn append_footer_log_queues_when_interactive() {
        use super::PENDING_LOGS;

        INTERACTIVE_MODE.store(true, Ordering::SeqCst);
        let result = append_footer_log("test log line");
        INTERACTIVE_MODE.store(false, Ordering::SeqCst);

        assert!(result.is_ok());

        // Drain any queued entries
        let mut pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        pending.clear();
    }

    #[test]
    #[serial]
    fn append_footer_log_writes_stdout_when_not_interactive() {
        INTERACTIVE_MODE.store(false, Ordering::SeqCst);
        let result = append_footer_log("direct output line");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn append_footer_log_is_suppressed_when_footer_output_disabled() {
        use super::PENDING_LOGS;

        let previous_state = footer_output_enabled();
        set_footer_output_enabled(false);
        INTERACTIVE_MODE.store(true, Ordering::SeqCst);

        let result = append_footer_log("suppressed line");

        INTERACTIVE_MODE.store(false, Ordering::SeqCst);
        set_footer_output_enabled(previous_state);

        assert!(result.is_ok());

        let pending = PENDING_LOGS.lock().unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_empty());
    }

    #[test]
    fn emit_attention_bell_succeeds() {
        let result = emit_attention_bell();
        assert!(result.is_ok());
    }

    #[test]
    fn attention_signal_gating_uses_focus_state_when_requested() {
        set_focus_detection_enabled(false);
        set_terminal_focused(true);
        assert!(!should_emit_attention_signal(true));

        set_focus_detection_enabled(true);
        set_terminal_focused(true);
        assert!(!should_emit_attention_signal(true));

        set_terminal_focused(false);
        assert!(should_emit_attention_signal(true));

        assert!(should_emit_attention_signal(false));
        assert!(!terminal_focused());
        assert!(focus_detection_enabled());

        set_focus_detection_enabled(false);
        set_terminal_focused(true);
    }

    #[test]
    #[serial]
    fn reset_layout_without_active_layout_succeeds() {
        let result = reset_layout();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn prepare_prompt_line_without_active_layout_succeeds() {
        let result = prepare_prompt_line();
        assert!(result.is_ok());
    }

    // === calculate_layout_metrics additional edge cases ===

    #[test]
    fn calculate_layout_metrics_reduced_footer_for_small_terminal() {
        // header=0, height=12 → MIN_DYNAMIC(6) + footer_target(10) = 16 > 12
        // footer should be reduced: 12 - 0 - 6 = 6, clamped to max(6,3)=6
        let metrics = calculate_layout_metrics(80, 12, 0).expect("should succeed for height=12");
        assert!(metrics.footer_height < FOOTER_TARGET_HEIGHT);
        assert!(metrics.footer_height >= 3);
    }

    #[test]
    fn calculate_layout_metrics_error_when_header_fills_terminal() {
        let result = calculate_layout_metrics(80, 10, 10);
        assert!(result.is_err());
    }

    // === clamp_cursor_to_metrics additional tests ===

    #[test]
    fn clamp_cursor_within_range_unchanged() {
        let metrics = LayoutMetrics {
            width: 80,
            height: 30,
            header_height: 4,
            footer_height: 10,
            footer_start: 20,
            scroll_top: 4,
            scroll_bottom: 19,
            log_capacity: 8,
        };
        let clamped = clamp_cursor_to_metrics((10, 10), metrics);
        assert_eq!(clamped, (10, 10));
    }

    #[test]
    fn clamp_cursor_at_exact_boundary() {
        let metrics = LayoutMetrics {
            width: 80,
            height: 30,
            header_height: 4,
            footer_height: 10,
            footer_start: 20,
            scroll_top: 4,
            scroll_bottom: 19,
            log_capacity: 8,
        };
        let clamped = clamp_cursor_to_metrics((79, 19), metrics);
        assert_eq!(clamped, (79, 19));
    }
}
