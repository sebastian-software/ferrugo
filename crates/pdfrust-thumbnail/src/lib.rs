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

/// Backend abstraction for document-level metadata inspection.
pub trait DocumentMetadataBackend {
    /// Stable backend name used in diagnostics and baseline metadata.
    fn backend_name(&self) -> &'static str;

    /// Inspects document metadata without rendering pixels.
    ///
    /// # Errors
    ///
    /// Implementations return [`ThumbnailError`] for stable caller-facing
    /// failure classes.
    fn inspect(&self, source: PdfSource<'_>) -> ThumbnailResult<DocumentMetadata>;
}

/// Document metadata shared by backend comparison harnesses.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentMetadata {
    /// Per-page metadata in document order.
    pub pages: Vec<PageMetadata>,
    /// Document information dictionary fields.
    pub info: DocumentInfo,
    /// Non-rendering catalog structure signals.
    pub structure: DocumentStructure,
    /// Document outline metadata.
    pub outlines: OutlineMetadata,
    /// Page label metadata.
    pub page_labels: PageLabelsMetadata,
    /// Tagged-PDF and accessibility-related metadata.
    pub accessibility: AccessibilityMetadata,
}

impl DocumentMetadata {
    /// Creates document metadata from resolved page metadata.
    #[must_use]
    pub fn new(pages: Vec<PageMetadata>) -> Self {
        Self {
            pages,
            info: DocumentInfo::default(),
            structure: DocumentStructure::default(),
            outlines: OutlineMetadata::default(),
            page_labels: PageLabelsMetadata::default(),
            accessibility: AccessibilityMetadata::default(),
        }
    }

    /// Returns the number of pages in the document.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns the first page size when the document has pages.
    #[must_use]
    pub fn first_page_size(&self) -> Option<PageSize> {
        self.pages.first().map(|page| page.size)
    }
}

/// Common PDF document information fields.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DocumentInfo {
    /// `/Title` from the document information dictionary.
    pub title: Option<String>,
    /// `/Author` from the document information dictionary.
    pub author: Option<String>,
    /// `/Subject` from the document information dictionary.
    pub subject: Option<String>,
    /// `/Keywords` from the document information dictionary.
    pub keywords: Option<String>,
    /// `/Creator` from the document information dictionary.
    pub creator: Option<String>,
    /// `/Producer` from the document information dictionary.
    pub producer: Option<String>,
    /// `/CreationDate` from the document information dictionary.
    pub creation_date: Option<String>,
    /// `/ModDate` from the document information dictionary.
    pub modification_date: Option<String>,
}

/// Non-rendering high-level document structure signals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DocumentStructure {
    /// Catalog contains XMP metadata through `/Metadata`.
    pub has_xmp_metadata: bool,
    /// Catalog contains `/MarkInfo`.
    pub has_mark_info: bool,
    /// Catalog contains `/StructTreeRoot`.
    pub has_struct_tree_root: bool,
    /// Catalog exposes named destinations through `/Dests` or `/Names /Dests`.
    pub has_named_destinations: bool,
    /// Document exposes at least one AcroForm signature field.
    pub has_signature_fields: bool,
    /// A signature dictionary exposes `/ByteRange` metadata.
    pub has_signature_byte_range: bool,
    /// Catalog names expose embedded files.
    pub has_embedded_files: bool,
    /// Catalog declares a portfolio `/Collection`.
    pub has_portfolio_collection: bool,
    /// Pages expose at least one file-attachment annotation.
    pub has_file_attachment_annotations: bool,
}

/// Bounded tagged-PDF and accessibility metadata signals.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AccessibilityMetadata {
    /// Catalog `/Lang` value when present.
    pub language: Option<String>,
    /// `/MarkInfo /Marked` value when present.
    pub mark_info_marked: Option<bool>,
    /// Structure tree root exposes a `/RoleMap` dictionary.
    pub has_role_map: bool,
    /// Number of structure element role names reached before the traversal budget.
    pub structure_role_count: usize,
    /// Structure tree references at least one marked-content sequence.
    pub has_marked_content_references: bool,
    /// Structure traversal stopped because the bounded item budget was reached.
    pub truncated: bool,
}

/// Outline tree metadata.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutlineMetadata {
    /// Catalog contains an `/Outlines` entry.
    pub has_outlines: bool,
    /// Number of outline items reached before the traversal budget.
    pub item_count: usize,
    /// Traversal stopped because the bounded item budget was reached.
    pub truncated: bool,
}

/// Page label metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PageLabelsMetadata {
    /// Resolved labels in page order up to the configured metadata budget.
    pub labels: Vec<PageLabel>,
    /// Label expansion stopped because the bounded label budget was reached.
    pub truncated: bool,
}

/// One resolved page label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageLabel {
    /// Zero-based page index.
    pub page_index: u32,
    /// Resolved display label.
    pub label: String,
}

/// Page metadata shared by backend comparison harnesses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMetadata {
    /// Zero-based page index.
    pub index: u32,
    /// Page size in PDF user-space units.
    pub size: PageSize,
}

/// Page size in PDF user-space units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    /// Page width.
    pub width: f64,
    /// Page height.
    pub height: f64,
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
    /// The document uses an unsupported feature with a stable diagnostic bucket.
    UnsupportedFeature(&'static str),
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

    /// Creates an unsupported-feature error with a stable internal bucket.
    #[must_use]
    pub const fn unsupported_feature(bucket: &'static str) -> Self {
        Self::UnsupportedFeature(bucket)
    }

    /// Returns the internal unsupported-feature bucket when one is available.
    #[must_use]
    pub const fn unsupported_feature_bucket(&self) -> Option<&'static str> {
        match self {
            Self::UnsupportedFeature(bucket) => Some(bucket),
            _ => None,
        }
    }

    /// Returns the stable high-level error class.
    #[must_use]
    pub const fn class(&self) -> ThumbnailErrorClass {
        match self {
            Self::Encrypted => ThumbnailErrorClass::Encrypted,
            Self::Malformed => ThumbnailErrorClass::Malformed,
            Self::Unsupported | Self::UnsupportedFeature(_) => ThumbnailErrorClass::Unsupported,
            Self::Timeout => ThumbnailErrorClass::Timeout,
            Self::Internal(_) => ThumbnailErrorClass::Internal,
        }
    }
}

/// Stable error classes for CLI output and baseline metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailErrorClass {
    /// The document is encrypted or password protected.
    Encrypted,
    /// The document is malformed or cannot be loaded as a PDF.
    Malformed,
    /// The document or request uses unsupported features.
    Unsupported,
    /// Rendering exceeded the configured timeout.
    Timeout,
    /// Backend failure not covered by a more specific stable class.
    Internal,
}

impl ThumbnailErrorClass {
    /// Returns the metadata-safe class name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Encrypted => "encrypted",
            Self::Malformed => "malformed",
            Self::Unsupported => "unsupported",
            Self::Timeout => "timeout",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ThumbnailErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for ThumbnailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encrypted => f.write_str("PDF is encrypted or password protected"),
            Self::Malformed => f.write_str("PDF is malformed"),
            Self::Unsupported => f.write_str("PDF feature is unsupported"),
            Self::UnsupportedFeature(bucket) => {
                write!(f, "PDF feature is unsupported ({bucket})")
            }
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
    fn error_class_should_be_stable() {
        assert_eq!(ThumbnailError::Timeout.class().as_str(), "timeout");
    }

    #[test]
    fn version_should_match_package_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
