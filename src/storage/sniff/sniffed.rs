use super::{FileError, MAX_VIDEO_SIZE};

/// A file's type, found from its first bytes: the client's own claim cannot
/// be trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sniffed {
    pub content_type: &'static str,
    pub extension: &'static str,
}

impl Sniffed {
    const PDF: Self = Self {
        content_type: "application/pdf",
        extension: "pdf",
    };
    const JPEG: Self = Self {
        content_type: "image/jpeg",
        extension: "jpg",
    };
    const PNG: Self = Self {
        content_type: "image/png",
        extension: "png",
    };
    const GIF: Self = Self {
        content_type: "image/gif",
        extension: "gif",
    };
    const WEBP: Self = Self {
        content_type: "image/webp",
        extension: "webp",
    };
    const WEBM: Self = Self {
        content_type: "video/webm",
        extension: "webm",
    };
    const QUICKTIME: Self = Self {
        content_type: "video/quicktime",
        extension: "mov",
    };
    const MP4: Self = Self {
        content_type: "video/mp4",
        extension: "mp4",
    };

    /// A PDF, JPEG or PNG.
    #[must_use]
    pub fn document(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(b"%PDF-") {
            Some(Self::PDF)
        } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Some(Self::JPEG)
        } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            Some(Self::PNG)
        } else {
            None
        }
    }

    /// A document, or any other image or video that is commonly sent as proof.
    #[must_use]
    pub fn evidence(bytes: &[u8]) -> Option<Self> {
        if let Some(found) = Self::document(bytes) {
            return Some(found);
        }
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            Some(Self::GIF)
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(Self::WEBP)
        } else if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            Some(Self::WEBM)
        } else if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
            // An MP4 family file; a QuickTime movie names its brand `qt`.
            if &bytes[8..10] == b"qt" {
                Some(Self::QUICKTIME)
            } else {
                Some(Self::MP4)
            }
        } else {
            None
        }
    }

    /// Checks a file sent as a receipt or a proof: a type that is accepted,
    /// and a size within the limit of that type (`document_limit` for a PDF
    /// or an image, [`MAX_VIDEO_SIZE`] for a video).
    ///
    /// # Errors
    ///
    /// [`FileError::UnsupportedFileType`] or [`FileError::TooLarge`].
    pub fn check_attachment(bytes: &[u8], document_limit: usize) -> Result<Self, FileError> {
        let found = Self::evidence(bytes).ok_or(FileError::UnsupportedFileType)?;
        if bytes.len() > found.size_limit(document_limit) {
            return Err(FileError::TooLarge);
        }
        Ok(found)
    }

    /// A video, which is converted for playback after it is stored.
    #[must_use]
    pub fn is_video(&self) -> bool {
        self.content_type.starts_with("video/")
    }

    /// The largest size this file may have: `document_limit` for a PDF or an
    /// image, [`MAX_VIDEO_SIZE`] for a video.
    #[must_use]
    pub fn size_limit(&self, document_limit: usize) -> usize {
        if self.is_video() {
            MAX_VIDEO_SIZE
        } else {
            document_limit
        }
    }
}
