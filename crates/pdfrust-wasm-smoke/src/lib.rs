//! WASM packaging smoke harness for the Rust-native renderer.

#![deny(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

use pdfrust_native::NativeBackend;
use pdfrust_thumbnail::{OutputFormat, PdfSource, Rgba, ThumbnailBackend, ThumbnailOptions};

const SMOKE_PDF: &[u8] = include_bytes!("../../../fixtures/generated/text-page.pdf");
const SMOKE_MAX_EDGE: u32 = 96;

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "wasm-smoke";

/// Successful WASM smoke-render metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmSmokeMetrics {
    /// Rendered thumbnail width in pixels.
    pub width: u32,
    /// Rendered thumbnail height in pixels.
    pub height: u32,
    /// Rendered RGBA byte length.
    pub output_bytes: usize,
}

/// Renders a tiny fixture through the low-memory native backend.
///
/// # Errors
///
/// Returns a stable static label when the native smoke render fails.
pub fn render_low_memory_smoke() -> Result<WasmSmokeMetrics, &'static str> {
    let thumbnail = NativeBackend::low_memory()
        .render(
            PdfSource::from_bytes(SMOKE_PDF),
            &ThumbnailOptions {
                page_index: 0,
                max_edge: SMOKE_MAX_EDGE,
                background: Rgba::WHITE,
                output_format: OutputFormat::Rgba,
                timeout: Duration::from_secs(5),
            },
        )
        .map_err(|_| "render")?;

    Ok(WasmSmokeMetrics {
        width: thumbnail.width,
        height: thumbnail.height,
        output_bytes: thumbnail.bytes.len(),
    })
}

/// Browser-callable smoke status.
///
/// Returns the rendered width and height packed into a non-zero `u32` on
/// success, or `0` on failure.
#[no_mangle]
pub extern "C" fn pdfrust_wasm_smoke_status() -> u32 {
    render_low_memory_smoke()
        .map(|metrics| (metrics.width << 16) | metrics.height)
        .unwrap_or(0)
}

/// Returns the stable role for this crate.
#[must_use]
pub const fn crate_role() -> &'static str {
    CRATE_ROLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_role_should_be_stable() {
        assert_eq!(crate_role(), "wasm-smoke");
    }

    #[test]
    fn wasm_smoke_should_render_low_memory_thumbnail_on_host() {
        let metrics = render_low_memory_smoke().expect("smoke fixture should render");

        assert!(metrics.width <= SMOKE_MAX_EDGE);
        assert!(metrics.height <= SMOKE_MAX_EDGE);
        assert_eq!(
            metrics.output_bytes,
            metrics.width as usize * metrics.height as usize * 4
        );
        assert_ne!(pdfrust_wasm_smoke_status(), 0);
    }
}
