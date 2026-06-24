//! Raster rendering primitives for the Rust-native renderer.

#![forbid(unsafe_code)]

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "render";

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

/// Returns the role of the lower-level content dependency.
#[must_use]
pub fn content_role() -> &'static str {
    pdfrust_content::crate_role()
}

/// Returns the bytes per pixel for the facade's initial RGBA output format.
#[must_use]
pub const fn facade_rgba_bytes_per_pixel() -> usize {
    pdfrust_thumbnail::PixelFormat::Rgba8.bytes_per_pixel()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "render");
    }

    #[test]
    fn render_should_depend_on_content() {
        assert_eq!(content_role(), "content");
    }

    #[test]
    fn render_should_use_thumbnail_facade_pixel_layout() {
        assert_eq!(facade_rgba_bytes_per_pixel(), 4);
    }
}
