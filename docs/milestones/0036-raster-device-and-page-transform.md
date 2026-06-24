# 0036: Raster Device And Page Transform

Status: todo
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

Empty until done.
