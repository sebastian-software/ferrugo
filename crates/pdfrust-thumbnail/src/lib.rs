//! Backend-neutral thumbnail generation facade.

use std::fmt;
use std::path::Path;
use std::time::Duration;

/// Default page rendered by thumbnail calls.
pub const DEFAULT_PAGE_INDEX: u32 = 0;

/// Default maximum width or height for generated thumbnails.
pub const DEFAULT_MAX_EDGE: u32 = 1024;

/// Default render timeout for one thumbnail.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Result alias for thumbnail operations.
pub type ThumbnailResult<T> = Result<T, ThumbnailError>;

/// Borrowed PDF input.
#[derive(Debug, Clone, Copy)]
pub enum PdfSource<'a> {
    /// Read PDF bytes from an existing in-memory buffer.
    Bytes(&'a [u8]),
    /// Read a PDF from the filesystem.
    File(&'a Path),
}

impl<'a> PdfSource<'a> {
    /// Creates a borrowed byte source.
    #[must_use]
    pub const fn from_bytes(bytes: &'a [u8]) -> Self {
        Self::Bytes(bytes)
    }

    /// Creates a borrowed file source.
    #[must_use]
    pub fn from_path(path: &'a Path) -> Self {
        Self::File(path)
    }
}

/// RGBA color with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl Rgba {
    /// Opaque white background.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
}

/// Requested output encoding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Raw RGBA bytes.
    #[default]
    Rgba,
    /// PNG encoded bytes.
    Png,
}

/// Pixel format for raw thumbnail buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Eight bits per channel, red-green-blue-alpha byte order.
    Rgba8,
}

impl PixelFormat {
    /// Bytes per pixel for this format.
    #[must_use]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 => 4,
        }
    }
}

/// Options for rendering a single thumbnail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbnailOptions {
    /// Zero-based page index.
    pub page_index: u32,
    /// Maximum width or height in pixels.
    pub max_edge: u32,
    /// Background used for transparent pages.
    pub background: Rgba,
    /// Requested output encoding.
    pub output_format: OutputFormat,
    /// Per-thumbnail timeout.
    pub timeout: Duration,
}

impl Default for ThumbnailOptions {
    fn default() -> Self {
        Self {
            page_index: DEFAULT_PAGE_INDEX,
            max_edge: DEFAULT_MAX_EDGE,
            background: Rgba::WHITE,
            output_format: OutputFormat::default(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// Rendered thumbnail bytes and layout metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thumbnail {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Number of bytes between adjacent rows.
    pub stride: usize,
    /// Raw pixel format.
    pub pixel_format: PixelFormat,
    /// Thumbnail bytes.
    pub bytes: Vec<u8>,
}

impl Thumbnail {
    /// Creates a raw RGBA thumbnail after validating dimensions and byte length.
    ///
    /// # Errors
    ///
    /// Returns [`ThumbnailError::Internal`] when dimensions overflow or the
    /// buffer length does not match the expected RGBA layout.
    pub fn rgba(width: u32, height: u32, bytes: Vec<u8>) -> ThumbnailResult<Self> {
        let stride = checked_stride(width, PixelFormat::Rgba8)?;
        let expected_len = stride
            .checked_mul(height as usize)
            .ok_or_else(|| ThumbnailError::internal("thumbnail dimensions overflow"))?;
        if bytes.len() != expected_len {
            return Err(ThumbnailError::internal("thumbnail buffer length mismatch"));
        }
        Ok(Self {
            width,
            height,
            stride,
            pixel_format: PixelFormat::Rgba8,
            bytes,
        })
    }
}

/// Backend abstraction for single-page thumbnail rendering.
pub trait ThumbnailBackend {
    /// Stable backend name used in diagnostics and baseline metadata.
    fn backend_name(&self) -> &'static str;

    /// Renders one thumbnail.
    ///
    /// # Errors
    ///
    /// Implementations return [`ThumbnailError`] for stable caller-facing
    /// failure classes.
    fn render(
        &self,
        source: PdfSource<'_>,
        options: &ThumbnailOptions,
    ) -> ThumbnailResult<Thumbnail>;
}

/// Stable thumbnail error taxonomy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailError {
    /// The document is encrypted or requires a password.
    Encrypted,
    /// The document is malformed or cannot be parsed.
    Malformed,
    /// The document uses unsupported features.
    Unsupported,
    /// Rendering exceeded the configured timeout.
    Timeout,
    /// Backend failure not covered by a more specific stable class.
    Internal(String),
}

impl ThumbnailError {
    /// Creates an internal error with an owned stable message.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl fmt::Display for ThumbnailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encrypted => f.write_str("PDF is encrypted or password protected"),
            Self::Malformed => f.write_str("PDF is malformed"),
            Self::Unsupported => f.write_str("PDF feature is unsupported"),
            Self::Timeout => f.write_str("thumbnail rendering timed out"),
            Self::Internal(message) => write!(f, "internal thumbnail error: {message}"),
        }
    }
}

impl std::error::Error for ThumbnailError {}

/// Returns the crate version compiled into this library.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn checked_stride(width: u32, pixel_format: PixelFormat) -> ThumbnailResult<usize> {
    (width as usize)
        .checked_mul(pixel_format.bytes_per_pixel())
        .ok_or_else(|| ThumbnailError::internal("thumbnail stride overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_should_match_phase_0_contract() {
        let options = ThumbnailOptions::default();

        assert_eq!(
            options,
            ThumbnailOptions {
                page_index: 0,
                max_edge: 1024,
                background: Rgba::WHITE,
                output_format: OutputFormat::Rgba,
                timeout: Duration::from_secs(5),
            }
        );
    }

    #[test]
    fn thumbnail_rgba_should_compute_stride() {
        let thumbnail = Thumbnail::rgba(2, 1, vec![0; 8]).expect("valid RGBA buffer");

        assert_eq!(thumbnail.stride, 8);
    }

    #[test]
    fn thumbnail_rgba_should_reject_mismatched_buffer_length() {
        let error = Thumbnail::rgba(2, 1, vec![0; 7]).expect_err("invalid RGBA buffer");

        assert_eq!(
            error.to_string(),
            "internal thumbnail error: thumbnail buffer length mismatch"
        );
    }

    #[test]
    fn encrypted_error_display_should_be_stable() {
        assert_eq!(
            ThumbnailError::Encrypted.to_string(),
            "PDF is encrypted or password protected"
        );
    }

    #[test]
    fn version_should_match_package_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
