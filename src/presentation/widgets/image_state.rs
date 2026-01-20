//! Image attachment state for message rendering.
//!
//! This module provides the presentation-layer wrapper that holds
//! both the decoded image and the ratatui-image protocol state.

use std::sync::Arc;

use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

use crate::domain::entities::{ImageId, ImageStatus};

/// Height of rendered images in terminal rows.
pub const IMAGE_HEIGHT: u16 = 20;

const MAX_IMAGE_WIDTH: u32 = 800;
const MAX_IMAGE_HEIGHT: u32 = 600;

/// Buffer size for loading images around visible area.
pub const LOAD_BUFFER: usize = 5;

/// Holds both the decoded image and the render-ready protocol.
pub struct ImageAttachment {
    pub id: ImageId,
    pub url: String,
    pub image: Option<Arc<image::DynamicImage>>,
    pub protocol: Option<StatefulProtocol>,
    pub last_width: u16,
    pub status: ImageStatus,
}

impl ImageAttachment {
    #[must_use]
    pub fn new(id: ImageId, url: String) -> Self {
        Self {
            id,
            url,
            image: None,
            protocol: None,
            last_width: 0,
            status: ImageStatus::NotStarted,
        }
    }

    #[must_use]
    pub fn from_attachment(attachment: &crate::domain::entities::Attachment) -> Option<Self> {
        if !attachment.is_image() {
            return None;
        }

        let id = ImageId::from_url(attachment.url());
        Some(Self::new(id, attachment.url().to_string()))
    }

    pub fn set_loaded(&mut self, image: Arc<image::DynamicImage>) {
        self.image = Some(image);
        self.status = ImageStatus::Ready;
        self.protocol = None;
        self.last_width = 0;
    }

    pub fn set_downloading(&mut self) {
        self.status = ImageStatus::Downloading;
    }

    pub fn set_failed(&mut self, error: String) {
        self.status = ImageStatus::Failed(error);
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.image.is_some() && self.status.is_ready()
    }

    #[must_use]
    pub const fn is_loading(&self) -> bool {
        self.status.is_loading()
    }

    #[must_use]
    pub const fn needs_load(&self) -> bool {
        self.status.is_not_started()
    }

    /// Updates the protocol if terminal width changed. Returns true if updated.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn update_protocol_if_needed(&mut self, picker: &Picker, terminal_width: u16) -> bool {
        let needs_update =
            self.image.is_some() && (self.protocol.is_none() || self.last_width != terminal_width);

        if !needs_update {
            return false;
        }

        let Some(ref image) = self.image else {
            return false;
        };

        let (width, height) = (image.width(), image.height());
        let resized_image = if width > MAX_IMAGE_WIDTH || height > MAX_IMAGE_HEIGHT {
            let scale_w = f64::from(MAX_IMAGE_WIDTH) / f64::from(width);
            let scale_h = f64::from(MAX_IMAGE_HEIGHT) / f64::from(height);
            let scale = scale_w.min(scale_h);

            let new_width = (f64::from(width) * scale) as u32;
            let new_height = (f64::from(height) * scale) as u32;

            image.resize(
                new_width,
                new_height,
                image::imageops::FilterType::CatmullRom,
            )
        } else {
            (**image).clone()
        };

        self.protocol = Some(picker.new_resize_protocol(resized_image));
        self.last_width = terminal_width;

        true
    }

    pub fn clear_protocol(&mut self) {
        self.protocol = None;
        self.last_width = 0;
    }

    #[must_use]
    pub const fn height(&self) -> u16 {
        if self.image.is_some() || self.status.is_loading() {
            IMAGE_HEIGHT
        } else {
            0
        }
    }
}

impl std::fmt::Debug for ImageAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageAttachment")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("has_image", &self.image.is_some())
            .field("has_protocol", &self.protocol.is_some())
            .field("last_width", &self.last_width)
            .field("status", &self.status)
            .finish()
    }
}

/// Manager for handling image attachments in messages.
/// Tracks which images need loading based on visible range.
pub struct ImageManager {
    /// The ratatui-image picker for protocol creation.
    picker: Picker,
    /// Current terminal width for resize tracking.
    current_width: u16,
}

impl ImageManager {
    /// Creates a new image manager with halfblocks (universal fallback).
    #[must_use]
    pub fn new() -> Self {
        // Use halfblocks as a safe universal fallback
        let picker = Picker::halfblocks();
        Self {
            picker,
            current_width: 0,
        }
    }

    /// Creates an image manager by querying the terminal.
    /// May block briefly during startup.
    #[must_use]
    pub fn from_query() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Self {
            picker,
            current_width: 0,
        }
    }

    /// Returns a reference to the picker.
    #[must_use]
    pub const fn picker(&self) -> &Picker {
        &self.picker
    }

    /// Updates the current terminal width.
    pub const fn set_width(&mut self, width: u16) {
        self.current_width = width;
    }

    /// Returns the current width.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.current_width
    }

    /// Updates protocols for visible images only.
    pub fn update_visible_protocols(&self, attachments: &mut [&mut ImageAttachment], width: u16) {
        for attachment in attachments {
            attachment.update_protocol_if_needed(&self.picker, width);
        }
    }

    /// Clears protocols for images outside the visible + buffer range.
    pub fn clear_distant_protocols(
        &self,
        attachments: &mut [&mut ImageAttachment],
        visible_start: usize,
        visible_end: usize,
    ) {
        let buffer_start = visible_start.saturating_sub(LOAD_BUFFER);
        let buffer_end = visible_end + LOAD_BUFFER;

        for (idx, attachment) in attachments.iter_mut().enumerate() {
            if idx < buffer_start || idx > buffer_end {
                attachment.clear_protocol();
            }
        }
    }

    /// Collects IDs of images that need loading within the visible + buffer range.
    #[must_use]
    pub fn collect_needed_loads(
        attachments: &[ImageAttachment],
        visible_start: usize,
        visible_end: usize,
    ) -> Vec<(ImageId, String)> {
        let buffer_start = visible_start.saturating_sub(LOAD_BUFFER);
        let buffer_end = visible_end + LOAD_BUFFER;

        attachments
            .iter()
            .enumerate()
            .filter(|(idx, attachment)| {
                *idx >= buffer_start && *idx <= buffer_end && attachment.needs_load()
            })
            .map(|(_, attachment)| (attachment.id.clone(), attachment.url.clone()))
            .collect()
    }
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_attachment_creation() {
        let id = ImageId::new("test");
        let attachment =
            ImageAttachment::new(id.clone(), "https://example.com/img.png".to_string());

        assert_eq!(attachment.id, id);
        assert!(!attachment.is_ready());
        assert!(attachment.needs_load());
    }

    #[test]
    fn test_image_attachment_loading_flow() {
        let id = ImageId::new("test");
        let mut attachment = ImageAttachment::new(id, "https://example.com/img.png".to_string());

        assert!(attachment.needs_load());

        attachment.set_downloading();
        assert!(attachment.is_loading());
        assert!(!attachment.needs_load());

        let img = Arc::new(image::DynamicImage::new_rgb8(100, 100));
        attachment.set_loaded(img);
        assert!(attachment.is_ready());
        assert!(!attachment.is_loading());
    }

    #[test]
    fn test_image_attachment_failure() {
        let id = ImageId::new("test");
        let mut attachment = ImageAttachment::new(id, "https://example.com/img.png".to_string());

        attachment.set_failed("Network error".to_string());
        assert!(attachment.status.is_failed());
        assert!(!attachment.is_ready());
    }

    #[test]
    fn test_image_manager_creation() {
        let manager = ImageManager::new();
        assert_eq!(manager.width(), 0);
    }

    #[test]
    fn test_collect_needed_loads() {
        let attachments = vec![
            ImageAttachment::new(ImageId::new("0"), "url0".to_string()),
            ImageAttachment::new(ImageId::new("1"), "url1".to_string()),
            ImageAttachment::new(ImageId::new("2"), "url2".to_string()),
            ImageAttachment::new(ImageId::new("3"), "url3".to_string()),
            ImageAttachment::new(ImageId::new("4"), "url4".to_string()),
        ];

        // Visible range 1-2, buffer should include 0-4 with LOAD_BUFFER=5
        let needed = ImageManager::collect_needed_loads(&attachments, 1, 2);
        assert!(needed.len() >= 2);
    }
}
