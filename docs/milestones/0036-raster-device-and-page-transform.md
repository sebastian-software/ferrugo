# 0036: Raster Device And Page Transform

Status: done
Phase: 3
Size: medium
Depends on: 0035

## Goal

Create the Rust-native raster device and page-to-pixel transform path.

## Scope

- Define an RGBA raster buffer with checked dimensions and stride.
- Map crop box, media box, rotation, background, and `max_edge` to device
  pixels.
- Add safe row and pixel access helpers.
- Keep low-level unsafe code out of the first implementation.

## Non-Goals

- Rasterize paths.
- Draw text or images.
- Optimize with SIMD.

## Deliverables

- Raster buffer type.
- Page transform calculation.
- Tests for dimensions, stride, and coordinate mapping.

## Acceptance Criteria

- The Rust renderer can allocate a correctly sized RGBA target for generated
  fixtures.
- Dimension overflow and oversized output requests fail safely.
- Output dimensions match PDFium for the current generated fixtures.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare dimensions against PDFium baselines.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added checked `RasterDimensions`, owned `RasterDevice`, safe row and pixel
  accessors, and typed `RasterError` / `RasterErrorKind` failures in
  `pdfrust-render`.
- Added `PageGeometry`, `PageRotation`, and `PageTransform` for mapping
  media/crop boxes, rotation, and `max_edge` into pixel-space target dimensions
  and a PDF-user-space-to-device matrix.
- The dimension policy matches the PDFium backend: scale down only when the
  rotated page's largest edge exceeds `max_edge`, round each dimension, and
  clamp to `1..=max_edge`.
- Added tests for stride and buffer overflow checks, background fill, safe
  row/pixel access, crop-box coordinate mapping, quarter-turn rotation,
  invalid inputs, and the PDFium-compatible `300x160` at `max-edge 256` result
  of `256x137`.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/text-page.pdf --max-edge 256 --output target/pdfrust-thumbnails/text-page-0036-256.png`
    produced a `256 x 137` RGBA PNG, matching the `PageTransform` test.
  - `cargo clippy --all-targets --all-features -- -D warnings`
