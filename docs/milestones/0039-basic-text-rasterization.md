# 0039: Basic Text Rasterization

Status: done
Phase: 3
Size: medium
Depends on: 0038

## Goal

Render enough text for common generated and office-like thumbnails to be
recognizable.

## Scope

- Choose and document the first font rendering dependency or internal strategy.
- Render simple embedded or base fonts used by the fixture set.
- Apply text matrix, font size, and fill color.
- Add reduced fixtures for browser-generated and office-like text PDFs.

## Non-Goals

- Full shaping.
- Full CMap and CID-keyed font coverage.
- Text extraction as a stable API.

## Deliverables

- Basic glyph rasterization path.
- Text fixture pixel comparisons.
- Documentation of unsupported font cases.

## Acceptance Criteria

- Generated text fixtures render visibly through the Rust backend.
- Common simple office/browser text PDFs are recognizable at thumbnail size.
- Unsupported font cases fail with typed errors or visible fallback behavior
  that is documented.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for text fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Chosen first strategy: internal 5x7 ASCII fallback font. This avoids adding a
  font dependency before the later dedicated font-program milestones while
  making generated ASCII text visibly recognizable in thumbnails.
- Added `rasterize_text` in `ferrugo-render` for positioned text display-list
  items. The fallback uses text origin, font size, and fill color; shaping,
  glyph outlines, kerning, CMaps, and non-ASCII text remain unsupported.
- Wired `ferrugo-native::NativeBackend::render` to resolve simple page
  `/Resources /Font` dictionaries into lightweight `FontResources`, build text
  display lists, and draw text after paths and images.
- Added tests for generated text fixture rasterization in both `ferrugo-render`
  and `ferrugo-native`.
- Fallback behavior: unsupported glyphs render as a visible placeholder glyph
  through the ASCII fallback path. This is intentional until 0042-0045 add real
  font loading, CMap handling, glyph outlines, and better positioning.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test -p ferrugo-render -p ferrugo-native`
  - `cargo test`
  - `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render fixtures/generated/text-page.pdf --max-edge 300 --output target/ferrugo-thumbnails/text-page-pdfium-0039.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/text-page.pdf --max-edge 300 --output target/ferrugo-thumbnails/text-page-native-0039.png`
  - Pixel comparison for those PNGs produced `dimensions=300x160 mae=12.082
    p95=92 max=255 native_nonwhite_pixels=2653`.
  - `cargo clippy --all-targets --all-features -- -D warnings`
