use std::io::{self, Write};
use crate::append_footer_log;

/// A writer that redirects formatted output to the terminal footer.
///
/// This writer implements `std::io::Write` and buffers incoming bytes,
/// emitting complete lines to the footer via `append_footer_log()`.
///
/// # Usage with tracing
///
/// ```rust,ignore
/// use tracing_subscriber::fmt;
/// use nettoolskit_ui::UiWriter;
///
/// let layer = fmt::layer()
///     .with_writer(|| UiWriter::default())
///     .compact();
/// ```
#[derive(Default)]
pub struct UiWriter {
    buffer: Vec<u8>,
}

impl UiWriter {
    /// Create a new UI writer.
    pub fn new() -> Self {
        Self::default()
    }

    fn emit_line(&self, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }

        let mut text = String::from_utf8_lossy(bytes).to_string();
        while text.ends_with('\n') || text.ends_with('\r') {
            text.pop();
        }

        if text.trim().is_empty() {
            return Ok(());
        }

        append_footer_log(&text)
    }
}

impl Write for UiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        while let Some(position) = self.buffer.iter().position(|&byte| byte == b'\n') {
            let chunk: Vec<u8> = self.buffer.drain(..=position).collect();
            self.emit_line(&chunk)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let chunk = self.buffer.split_off(0);
            self.emit_line(&chunk)?;
        }
        Ok(())
    }
}