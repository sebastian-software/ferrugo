//! Rust-native backend adapter for the thumbnail facade.

#![forbid(unsafe_code)]

use pdfrust_thumbnail::{PdfSource, Thumbnail, ThumbnailBackend, ThumbnailError, ThumbnailOptions};

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
}

impl ThumbnailBackend for NativeBackend {
    fn backend_name(&self) -> &'static str {
        "rust-native"
    }

    fn render(
        &self,
        _source: PdfSource<'_>,
        _options: &ThumbnailOptions,
    ) -> Result<Thumbnail, ThumbnailError> {
        Err(ThumbnailError::Unsupported)
    }
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
    fn native_backend_should_depend_on_object_and_render_layers() {
        assert_eq!(object_role(), "object");
        assert_eq!(render_role(), "render");
    }
}
