//! Rust-native backend adapter for the thumbnail facade.

#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::fs;

use pdfrust_object::{load_classic_document, load_modern_document, PageTree};
use pdfrust_syntax::PdfBytes;
use pdfrust_thumbnail::{
    DocumentMetadata, DocumentMetadataBackend, PageMetadata as ThumbnailPageMetadata, PageSize,
    PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions,
};

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "native-backend";

/// Rust-native thumbnail backend.
///
/// The backend is intentionally a placeholder until the parser, object model,
/// content interpreter, and rasterizer land in later milestones.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NativeBackend;

impl NativeBackend {
    /// Creates a new Rust-native backend placeholder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the stable backend name.
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        "rust-native"
    }
}

impl ThumbnailBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        Self::backend_name(self)
    }

    fn render(
        &self,
        _source: PdfSource<'_>,
        _options: &ThumbnailOptions,
    ) -> Result<Thumbnail, ThumbnailError> {
        Err(ThumbnailError::Unsupported)
    }
}

impl DocumentMetadataBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        Self::backend_name(self)
    }

    fn inspect(&self, source: PdfSource<'_>) -> Result<DocumentMetadata, ThumbnailError> {
        let bytes = load_source(source)?;
        inspect_bytes(&bytes)
    }
}

fn load_source(source: PdfSource<'_>) -> Result<Cow<'_, [u8]>, ThumbnailError> {
    match source {
        PdfSource::Bytes(bytes) => Ok(Cow::Borrowed(bytes)),
        PdfSource::File(path) => fs::read(path)
            .map(Cow::Owned)
            .map_err(|_| ThumbnailError::Malformed),
    }
}

fn inspect_bytes(bytes: &[u8]) -> Result<DocumentMetadata, ThumbnailError> {
    let input = PdfBytes::new(bytes);
    match load_modern_document(input).and_then(|document| document.page_tree()) {
        Ok(page_tree) => metadata_from_page_tree(&page_tree),
        Err(_) => load_classic_document(input)
            .and_then(|document| document.page_tree())
            .map_err(|_| ThumbnailError::Malformed)
            .and_then(|page_tree| metadata_from_page_tree(&page_tree)),
    }
}

fn metadata_from_page_tree(page_tree: &PageTree) -> Result<DocumentMetadata, ThumbnailError> {
    let pages = page_tree
        .pages()
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let index = u32::try_from(index)
                .map_err(|_| ThumbnailError::internal("page index exceeds u32"))?;
            let size = page.size();
            Ok(ThumbnailPageMetadata {
                index,
                size: PageSize {
                    width: size.width,
                    height: size.height,
                },
            })
        })
        .collect::<Result<Vec<_>, ThumbnailError>>()?;
    Ok(DocumentMetadata::new(pages))
}

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the object-model dependency.
#[must_use]
pub fn object_role() -> &'static str {
    pdfrust_object::crate_role()
}

/// Returns the role of the render dependency.
#[must_use]
pub fn render_role() -> &'static str {
    pdfrust_render::crate_role()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "native-backend");
    }

    #[test]
    fn native_backend_name_should_be_backend_neutral() {
        assert_eq!(NativeBackend::new().backend_name(), "rust-native");
    }

    #[test]
    fn native_backend_should_start_as_unsupported() {
        let error = NativeBackend::new()
            .render(
                PdfSource::from_bytes(b"%PDF-1.7"),
                &ThumbnailOptions::default(),
            )
            .expect_err("placeholder backend should not render yet");

        assert_eq!(error, ThumbnailError::Unsupported);
    }

    #[test]
    fn native_backend_should_inspect_generated_fixture_metadata() {
        let bytes = include_bytes!("../../../fixtures/generated/text-page.pdf");
        let metadata =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(bytes))
                .expect("generated fixture should inspect");

        assert_eq!(metadata.page_count(), 1);
        assert_eq!(
            metadata.first_page_size(),
            Some(PageSize {
                width: 300.0,
                height: 160.0,
            })
        );
    }

    #[test]
    fn native_backend_should_map_invalid_input_to_malformed() {
        let error =
            DocumentMetadataBackend::inspect(&NativeBackend::new(), PdfSource::from_bytes(b"nope"))
                .expect_err("invalid PDF should fail");

        assert_eq!(error, ThumbnailError::Malformed);
    }

    #[test]
    fn native_backend_should_depend_on_object_and_render_layers() {
        assert_eq!(object_role(), "object");
        assert_eq!(render_role(), "render");
    }
}
