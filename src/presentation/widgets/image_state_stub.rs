//! Image attachment state for message rendering (stub when image feature disabled).

use crate::domain::entities::{ImageId, ImageStatus};

pub const MAX_IMAGE_HEIGHT: u16 = 20;
pub const LOAD_BUFFER: usize = 5;

/// Stub ImageAttachment when image feature is disabled.
pub struct ImageAttachment {
    pub id: ImageId,
    pub url: String,
    pub status: ImageStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub protocol: Option<()>,
}

impl ImageAttachment {
    #[must_use]
    pub fn new(id: ImageId, url: String, width: Option<u32>, height: Option<u32>) -> Self {
        Self {
            id,
            url,
            status: ImageStatus::NotStarted,
            width,
            height,
            protocol: None,
        }
    }

    #[must_use]
    pub fn from_attachment(attachment: &crate::domain::entities::Attachment) -> Option<Self> {
        if !attachment.is_image() {
            return None;
        }

        let id = ImageId::from_url(&attachment.url);
        Some(Self::new(
            id,
            attachment.url.clone(),
            attachment.width,
            attachment.height,
        ))
    }

    pub fn set_downloading(&mut self) {
        self.status = ImageStatus::Downloading;
    }

    pub fn set_failed(&mut self, error: String) {
        self.status = ImageStatus::Failed(error);
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        false
    }

    #[must_use]
    pub const fn is_loading(&self) -> bool {
        self.status.is_loading()
    }

    #[must_use]
    pub const fn needs_load(&self) -> bool {
        false
    }

    pub fn clear_protocol(&mut self) {}

    #[must_use]
    pub fn height(&self, _width: u16) -> u16 {
        0
    }

    #[must_use]
    pub fn width(&self, _width: u16) -> u16 {
        0
    }
}

impl std::fmt::Debug for ImageAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageAttachment")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

/// Stub ImageManager when image feature is disabled.
pub struct ImageManager;

impl ImageManager {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn halfblocks() -> Self {
        Self
    }

    #[deprecated(since = "0.2.0", note = "use `new()` instead")]
    #[must_use]
    pub fn from_query() -> Self {
        Self::new()
    }

    #[must_use]
    pub const fn width(&self) -> u16 {
        0
    }

    pub const fn set_width(&mut self, _width: u16) {}

    pub fn update_visible_protocols(&self, _attachments: &mut [&mut ImageAttachment]) {}

    pub fn clear_distant_protocols(
        &self,
        _attachments: &mut [&mut ImageAttachment],
        _visible_start: usize,
        _visible_end: usize,
    ) {
    }

    #[must_use]
    pub fn collect_needed_loads(
        _attachments: &[ImageAttachment],
        _visible_start: usize,
        _visible_end: usize,
    ) -> Vec<(ImageId, String)> {
        Vec::new()
    }
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}
