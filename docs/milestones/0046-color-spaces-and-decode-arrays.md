# 0046: Color Spaces And Decode Arrays

Status: done
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

- Implemented DeviceGray, DeviceRGB, DeviceCMYK, Indexed DeviceGray/RGB, and
  CalGray/CalRGB fallback handling for image resources.
- Added stack-backed image Decode array parsing for supported process and
  Indexed color spaces, applying ranges in-place after stream decoding.
- ICCBased and non-process color spaces remain explicit typed unsupported
  cases; full color management remains a non-goal.
- Added deterministic `cmyk-image.pdf` and `indexed-image.pdf` fixtures with
  native backend coverage.
- Validation: `cargo fmt --check`, `cargo check`, `cargo test`,
  `cargo test -p pdfrust-render -p pdfrust-native`,
  `cargo clippy --all-targets --all-features -- -D warnings`, native CLI
  smokes for `cmyk-image.pdf` and `indexed-image.pdf`, and PDFium/native pixel
  comparisons:
  - `cmyk-image.pdf`: `120x120`, MAE `15.306`, max `109`,
    `native_nonwhite_pixels=6400`.
  - `indexed-image.pdf`: `120x120`, MAE `0.000`, max `0`,
    `native_nonwhite_pixels=4800`.

## Progress Notes

- Added DeviceCMYK image color-space recognition and allocation-light
  subtractive CMYK-to-RGB sampling in the rasterizer.
- Added stack-backed image Decode array parsing for DeviceGray, DeviceRGB, and
  DeviceCMYK samples, applying supported decode ranges in-place after stream
  decoding.
- Added Indexed color-space parsing for DeviceGray and DeviceRGB lookup tables,
  keeping index samples compact and resolving palette colors during raster
  sampling.
- Kept unsupported non-process color spaces typed; `Separation` remains an
  explicit unsupported color-space error.
