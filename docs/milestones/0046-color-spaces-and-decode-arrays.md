# 0046: Color Spaces And Decode Arrays

Status: in-progress
Phase: 6
Size: medium
Depends on: 0045

## Goal

Render common non-RGB color spaces and image decode arrays well enough for
typical thumbnails.

## Scope

- Support DeviceGray, DeviceRGB, DeviceCMYK, Indexed, and calibrated fallback
  behavior.
- Apply image decode arrays for supported image color spaces.
- Add explicit ICCBased handling policy with graceful fallback or unsupported
  errors.
- Keep conversions allocation-light and streaming-friendly where possible.

## Non-Goals

- Full color-managed output.
- Printer-grade CMYK accuracy.
- Spot color and overprint fidelity.

## Deliverables

- Color conversion module.
- Decode-array tests.
- Fixture comparisons for CMYK and Indexed image PDFs.

## Acceptance Criteria

- Common CMYK and Indexed fixtures render with recognizable colors.
- Unsupported color spaces fail with typed diagnostics.
- Color conversion avoids per-pixel heap allocation.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for color fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.

## Progress Notes

- Added DeviceCMYK image color-space recognition and allocation-light
  subtractive CMYK-to-RGB sampling in the rasterizer.
- Kept unsupported non-process color spaces typed; `Separation` remains an
  explicit unsupported color-space error.
