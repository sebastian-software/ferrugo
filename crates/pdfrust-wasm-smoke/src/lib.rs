//! WASM packaging smoke harness for the Rust-native renderer.

#![deny(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

use pdfrust_native::NativeBackend;
use pdfrust_thumbnail::{
    AnnotationMode, OutputFormat, PdfSource, Rgba, ThumbnailBackend, ThumbnailOptions,
};

const SMOKE_PDF: &[u8] = b"%PDF-1.4\n\
%\xE2\xE3\xCF\xD3\n\
1 0 obj\n\
<< /Length 55 >>\n\
stream\n\
BT /F1 24 Tf 40 90 Td (pdfrust thumbnail fixture) Tj ET\n\
endstream\n\
endobj\n\
2 0 obj\n\
<< /Type /Page /Parent 3 0 R /MediaBox [0 0 300 160] /Resources << /Font << /F1 4 0 R >> >> /Contents 1 0 R >>\n\
endobj\n\
3 0 obj\n\
<< /Type /Pages /Kids [2 0 R] /Count 1 >>\n\
endobj\n\
4 0 obj\n\
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n\
endobj\n\
5 0 obj\n\
<< /Type /Catalog /Pages 3 0 R >>\n\
endobj\n\
xref\n\
0 6\n\
0000000000 65535 f \n\
0000000015 00000 n \n\
0000000120 00000 n \n\
0000000246 00000 n \n\
0000000303 00000 n \n\
0000000373 00000 n \n\
trailer\n\
<< /Size 6 /Root 5 0 R >>\n\
startxref\n\
422\n\
%%EOF\n";
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
                annotation_mode: AnnotationMode::Screen,
                form_appearance_mode: pdfrust_thumbnail::FormAppearanceMode::DocumentState,
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
