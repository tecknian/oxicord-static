//! Clipboard service using xclip/wl-copy shell commands (static-friendly).

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn};

/// Clipboard service implementation using shell commands.
#[derive(Clone)]
pub struct ClipboardService {
    has_xclip: bool,
    has_wl_copy: bool,
    warned: std::sync::Arc<AtomicBool>,
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardService {
    #[must_use]
    pub fn new() -> Self {
        let has_xclip = Self::check_cmd("xclip", &["-version"]);
        let has_wl_copy = Self::check_cmd("wl-copy", &["--version"]);

        let warned = std::sync::Arc::new(AtomicBool::new(false));

        if !has_xclip && !has_wl_copy {
            warn!("Clipboard tools not found. Install xclip (X11) or wl-clipboard (Wayland) for clipboard support.");
            warned.store(true, Ordering::SeqCst);
        } else {
            debug!(
                xclip = has_xclip,
                wl_copy = has_wl_copy,
                "Clipboard tools detected"
            );
        }

        Self {
            has_xclip,
            has_wl_copy,
            warned,
        }
    }

    fn check_cmd(cmd: &str, args: &[&str]) -> bool {
        Command::new(cmd).args(args).output().is_ok()
    }

    fn has_clipboard(&self) -> bool {
        self.has_xclip || self.has_wl_copy
    }

    fn warn_once(&self) {
        if !self.warned.load(Ordering::SeqCst) {
            warn!("Clipboard operation failed. Is xclip or wl-copy installed?");
            self.warned.store(true, Ordering::SeqCst);
        }
    }

    /// Set text to clipboard.
    pub fn set_text(&self, text: impl Into<String>) {
        let text = text.into();
        if !self.has_clipboard() {
            self.warn_once();
            return;
        }

        tokio::task::spawn_blocking(move || {
            // Try xclip first
            if Self::try_xclip_text(&text).is_ok() {
                return;
            }
            // Try wl-copy
            if Self::try_wl_copy_text(&text).is_ok() {
                return;
            }
        });
    }

    fn try_xclip_text(text: &str) -> Result<(), ()> {
        let mut child = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|_| ())?;
        }

        child.wait().map_err(|_| ())?;
        Ok(())
    }

    fn try_wl_copy_text(text: &str) -> Result<(), ()> {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).map_err(|_| ())?;
        }

        child.wait().map_err(|_| ())?;
        Ok(())
    }

    /// Get text from clipboard.
    #[must_use]
    pub fn get_text(&self) -> Option<String> {
        if !self.has_clipboard() {
            self.warn_once();
            return None;
        }

        // Try xclip first
        if let Ok(text) = Self::try_xclip_output() {
            return Some(text);
        }

        // Try wl-paste
        if let Ok(text) = Self::try_wl_paste() {
            return Some(text);
        }

        None
    }

    fn try_xclip_output() -> Result<String, ()> {
        let output = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
            .map_err(|_| ())?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|_| ())
        } else {
            Err(())
        }
    }

    fn try_wl_paste() -> Result<String, ()> {
        let output = Command::new("wl-paste")
            .args(&["--no-newline"])
            .output()
            .map_err(|_| ())?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|_| ())
        } else {
            Err(())
        }
    }

    /// Set binary data to clipboard with MIME type.
    pub fn set_binary(&self, data: Vec<u8>, mime_type: &str) {
        if !self.has_clipboard() {
            self.warn_once();
            return;
        }

        let mime = mime_type.to_string();
        tokio::task::spawn_blocking(move || {
            // Try xclip with MIME type
            if Self::try_xclip_binary(&data, &mime).is_ok() {
                return;
            }
            // Try wl-copy with MIME type
            if Self::try_wl_copy_binary(&data, &mime).is_ok() {
                return;
            }
        });
    }

    fn try_xclip_binary(data: &[u8], mime: &str) -> Result<(), ()> {
        let mut child = Command::new("xclip")
            .args(&["-selection", "clipboard", "-t", mime])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data).map_err(|_| ())?;
        }

        child.wait().map_err(|_| ())?;
        Ok(())
    }

    fn try_wl_copy_binary(data: &[u8], mime: &str) -> Result<(), ()> {
        let mut child = Command::new("wl-copy")
            .args(&["--type", mime])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data).map_err(|_| ())?;
        }

        child.wait().map_err(|_| ())?;
        Ok(())
    }

    /// Get binary data from clipboard.
    /// Returns (data, mime_type) if available.
    #[must_use]
    pub fn get_binary(&self) -> Option<(Vec<u8>, String)> {
        if !self.has_clipboard() {
            self.warn_once();
            return None;
        }

        // Try xclip with common image types
        for mime in &["image/png", "image/jpeg", "image/webp", "text/uri-list"] {
            if let Ok(data) = Self::try_xclip_binary_output(mime) {
                return Some((data, mime.to_string()));
            }
        }

        // Try wl-paste
        if let Ok((data, mime)) = Self::try_wl_paste_binary() {
            return Some((data, mime));
        }

        None
    }

    fn try_xclip_binary_output(mime: &str) -> Result<Vec<u8>, ()> {
        let output = Command::new("xclip")
            .args(&["-selection", "clipboard", "-t", mime, "-o"])
            .output()
            .map_err(|_| ())?;

        if output.status.success() && !output.stdout.is_empty() {
            Ok(output.stdout)
        } else {
            Err(())
        }
    }

    fn try_wl_paste_binary() -> Result<(Vec<u8>, String), ()> {
        // First try to get MIME type
        let type_output = Command::new("wl-paste")
            .args(&["--list-types"])
            .output()
            .map_err(|_| ())?;

        if !type_output.status.success() {
            return Err(());
        }

        let mime = String::from_utf8_lossy(&type_output.stdout);
        let mime = mime.lines().next().unwrap_or("application/octet-stream");

        // Get the actual data
        let output = Command::new("wl-paste").output().map_err(|_| ())?;

        if output.status.success() {
            Ok((output.stdout, mime.to_string()))
        } else {
            Err(())
        }
    }
}

/// Stub ImageData type for compatibility with existing code.
pub mod stub_image {
    pub struct ImageData {
        pub width: usize,
        pub height: usize,
        pub bytes: std::borrow::Cow<'static, [u8]>,
    }
}
