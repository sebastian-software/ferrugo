# 0038: Image Rasterization And Alpha

Status: todo
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

Empty until done.
