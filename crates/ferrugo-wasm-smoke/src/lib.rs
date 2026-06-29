//! WASM packaging smoke harness for the Rust-native renderer.

#![deny(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

use ferrugo_native::NativeBackend;
use ferrugo_thumbnail::{
    AnnotationMode, OutputFormat, PdfSource, Rgba, ThumbnailBackend, ThumbnailOptions,
};

const SMOKE_MAX_EDGE: u32 = 96;

struct WasmSmokeFixture {
    name: &'static str,
    pdf: &'static [u8],
}

const SMOKE_FIXTURES: &[WasmSmokeFixture] = &[
    WasmSmokeFixture {
        name: "text-page",
        pdf: include_bytes!("../../../fixtures/generated/text-page.pdf"),
    },
    WasmSmokeFixture {
        name: "browser-print",
        pdf: include_bytes!("../../../fixtures/generated/browser-chromium-article-print.pdf"),
    },
    WasmSmokeFixture {
        name: "mobile-scan",
        pdf: include_bytes!("../../../fixtures/generated/mobile-cropped-photo-scan.pdf"),
    },
    WasmSmokeFixture {
        name: "form-preview",
        pdf: include_bytes!("../../../fixtures/generated/acroform-text-field.pdf"),
    },
    WasmSmokeFixture {
        name: "invoice-preview",
        pdf: include_bytes!("../../../fixtures/generated/business-invoice-dense.pdf"),
    },
];

/// Stable crate role used by architecture smoke tests and documentation.
pub const CRATE_ROLE: &str = "wasm-smoke";

/// Successful WASM smoke-render metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WasmSmokeMetrics {
    /// Number of embedded preview fixtures rendered by the smoke suite.
    pub fixture_count: u32,
    /// Rendered thumbnail width in pixels.
    pub width: u32,
    /// Rendered thumbnail height in pixels.
    pub height: u32,
    /// Rendered RGBA byte length.
    pub output_bytes: usize,
    /// Total RGBA bytes produced across the preview smoke suite.
    pub total_output_bytes: usize,
    /// Largest single thumbnail byte length produced by the preview smoke suite.
    pub max_output_bytes: usize,
}

/// Renders the browser-preview smoke fixtures through the low-memory backend.
///
/// # Errors
///
/// Returns a stable static label when the native smoke render fails.
pub fn render_low_memory_smoke() -> Result<WasmSmokeMetrics, &'static str> {
    let backend = NativeBackend::low_memory();
    let mut first_dimensions = None;
    let mut total_output_bytes = 0usize;
    let mut max_output_bytes = 0usize;

    for fixture in SMOKE_FIXTURES {
        let thumbnail = backend
            .render(PdfSource::from_bytes(fixture.pdf), &thumbnail_options())
            .map_err(|_| fixture.name)?;

        let output_bytes = thumbnail.bytes.len();
        first_dimensions.get_or_insert((thumbnail.width, thumbnail.height, output_bytes));
        total_output_bytes = total_output_bytes
            .checked_add(output_bytes)
            .ok_or("output-bytes")?;
        max_output_bytes = max_output_bytes.max(output_bytes);
    }

    let Some((width, height, output_bytes)) = first_dimensions else {
        return Err("fixtures");
    };

    Ok(WasmSmokeMetrics {
        fixture_count: SMOKE_FIXTURES.len() as u32,
        width,
        height,
        output_bytes,
        total_output_bytes,
        max_output_bytes,
    })
}

fn thumbnail_options() -> ThumbnailOptions {
    ThumbnailOptions {
        page_index: 0,
        max_edge: SMOKE_MAX_EDGE,
        background: Rgba::WHITE,
        output_format: OutputFormat::Rgba,
        timeout: Duration::from_secs(5),
        annotation_mode: AnnotationMode::Screen,
        form_appearance_mode: ferrugo_thumbnail::FormAppearanceMode::DocumentState,
    }
}

/// Browser-callable smoke status.
///
/// Returns the rendered width and height packed into a non-zero `u32` on
/// success, or `0` on failure.
#[no_mangle]
pub extern "C" fn ferrugo_wasm_smoke_status() -> u32 {
    render_low_memory_smoke()
        .map(|metrics| (metrics.width << 16) | metrics.height)
        .unwrap_or(0)
}

/// Returns the number of embedded browser-preview fixtures in the smoke suite.
#[no_mangle]
pub extern "C" fn ferrugo_wasm_smoke_fixture_count() -> u32 {
    SMOKE_FIXTURES.len() as u32
}

/// Renders the smoke suite and returns total RGBA bytes, or `0` on failure.
#[no_mangle]
pub extern "C" fn ferrugo_wasm_smoke_total_output_bytes() -> u32 {
    render_low_memory_smoke()
        .ok()
        .and_then(|metrics| u32::try_from(metrics.total_output_bytes).ok())
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

        assert_eq!(metrics.fixture_count, SMOKE_FIXTURES.len() as u32);
        assert!(metrics.width <= SMOKE_MAX_EDGE);
        assert!(metrics.height <= SMOKE_MAX_EDGE);
        assert_eq!(
            metrics.output_bytes,
            metrics.width as usize * metrics.height as usize * 4
        );
        assert!(metrics.total_output_bytes >= metrics.output_bytes);
        assert!(metrics.max_output_bytes >= metrics.output_bytes);
        assert_ne!(ferrugo_wasm_smoke_status(), 0);
        assert_eq!(
            ferrugo_wasm_smoke_fixture_count(),
            SMOKE_FIXTURES.len() as u32
        );
        assert_eq!(
            ferrugo_wasm_smoke_total_output_bytes() as usize,
            metrics.total_output_bytes
        );
    }
}
