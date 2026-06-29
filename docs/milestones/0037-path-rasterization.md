# 0037: Path Rasterization

Status: done
Phase: 3
Size: medium
Depends on: 0036

## Goal

Rasterize basic filled and stroked paths into RGBA thumbnails.

## Scope

- Flatten line and curve path segments.
- Fill nonzero and even-odd paths.
- Stroke simple paths with width and joins sufficient for generated fixtures.
- Add antialiasing strategy for thumbnail output.

## Non-Goals

- Gradients, patterns, transparency groups, or blend modes.
- Perfect PDFium parity for all stroke joins.
- SIMD optimization.

## Deliverables

- Path rasterizer.
- Pixel tests for generated vector fixtures.
- Tolerance policy for path rendering differences.

## Acceptance Criteria

- Simple vector PDFs render through the Rust backend to non-empty RGBA output.
- Pixel comparisons against PDFium pass within documented tolerance.
- Path complexity limits prevent excessive memory and CPU use.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for vector fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `rasterize_paths` and `PathRasterOptions` in `ferrugo-render`.
- The path rasterizer flattens lines and cubics into bounded line segments,
  fills nonzero and even-odd paths, strokes simple line segments, and paints
  opaque DeviceGray/DeviceRGB colors into RGBA output with fixed supersampling.
- Added `PathComplexityOverflow` and `InvalidSupersampling` raster errors to
  fail safely on excessive path complexity or invalid raster options.
- Wired `ferrugo-native::NativeBackend::render` for simple path-only Classic
  PDFs by decoding `/Contents`, building a path display list, applying
  `PageTransform`, and returning a `Thumbnail`.
- Added `ferrugo-cli render-native` for direct native PNG smoke renders.
- Tolerance policy for this milestone: generated vector fixtures must match
  PDFium dimensions exactly, produce non-empty native RGBA output, and keep mean
  absolute channel error at or below 20 with p95 channel error at or below 64.
  Edge antialiasing differences may have higher max-channel outliers until the
  rasterizer gains PDFium-equivalent coverage rules.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render fixtures/generated/vector-paths.pdf --max-edge 220 --output target/ferrugo-thumbnails/vector-paths-pdfium-0037.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/vector-paths.pdf --max-edge 220 --output target/ferrugo-thumbnails/vector-paths-native-0037.png`
  - Pixel comparison for those PNGs produced `dimensions=220x180 mae=0.171
    p95=0 max=229 native_nonwhite_pixels=5134`.
  - `cargo clippy --all-targets --all-features -- -D warnings`
