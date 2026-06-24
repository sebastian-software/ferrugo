# 0038: Image Rasterization And Alpha

Status: done
Phase: 3
Size: medium
Depends on: 0037

## Goal

Draw decoded image XObjects into the Rust raster buffer with basic alpha
handling.

## Scope

- Implement nearest-neighbor or bilinear sampling for thumbnail-sized images.
- Apply image transforms from the display list.
- Composite DeviceRGB and DeviceGray image pixels over the page background.
- Add simple alpha or mask handling only where fixtures require it.

## Non-Goals

- Full ICC color management.
- JPX, CCITT, or JBIG2 decoding.
- Advanced blend modes.

## Deliverables

- Image drawing path.
- Sampling tests.
- Pixel comparisons for generated image fixtures.

## Acceptance Criteria

- Generated image PDFs render recognizably with the Rust backend.
- Image output dimensions and placement match PDFium within tolerance.
- Unsupported color spaces and filters fail explicitly.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for image fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `rasterize_images` in `pdfrust-render` for decoded Image XObject
  display-list items.
- Image rasterization composes the page transform with the image placement
  matrix, inverts it for nearest-neighbor unit-square sampling, and draws
  opaque `DeviceRGB` and `DeviceGray` samples into the RGBA raster.
- Added `Matrix::inverse` and a typed `SingularImageTransform` raster error for
  non-invertible image placements.
- Wired `pdfrust-native::NativeBackend::render` to resolve page
  `/Resources /XObject` image dictionaries, build image display lists, and draw
  images after path rasterization.
- Added generated `fixtures/generated/image-xobject.pdf` through
  `scripts/generate_fixtures.py`.
- Added tests for image rasterization quadrants and native backend image
  rendering.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test -p pdfrust-render -p pdfrust-native`
  - `cargo test`
  - `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- render fixtures/generated/image-xobject.pdf --max-edge 120 --output target/pdfrust-thumbnails/image-xobject-pdfium-0038.png`
  - `cargo run -p pdfrust-cli -- render-native fixtures/generated/image-xobject.pdf --max-edge 120 --output target/pdfrust-thumbnails/image-xobject-native-0038.png`
  - Pixel comparison for those PNGs produced `dimensions=120x120 mae=0.000
    p95=0 max=0 native_nonwhite_pixels=4096`.
  - `cargo clippy --all-targets --all-features -- -D warnings`
