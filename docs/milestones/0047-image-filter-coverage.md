# 0047: Image Filter Coverage

Status: in-progress
Phase: 6
Size: medium
Depends on: 0046

## Goal

Decode the image filters that appear most often in typical PDFs.

## Scope

- Add `DCTDecode` support through a documented Rust dependency.
- Evaluate and implement the first safe CCITT or JPX policy from corpus data.
- Apply predictor handling for PNG-style image data where required.
- Bound decoded image dimensions and total pixel memory.

## Non-Goals

- Implement every PDF image codec from scratch.
- Support unsafe native codec integrations without an explicit decision record.
- Optimize high-resolution print rendering.

## Deliverables

- Image filter support extension.
- Codec decision notes.
- Tests for valid images, malformed images, and memory limits.

## Acceptance Criteria

- JPEG-backed image PDFs render through the native backend.
- Unsupported codecs are reported consistently.
- Oversized image inputs are rejected before unbounded allocation.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-heavy corpus comparisons against PDFium.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice adds `DCTDecode` support for 8-bit DeviceRGB and
  DeviceGray Image XObjects through the pure Rust `zune-jpeg` decoder with
  default native SIMD features disabled.
- Codec dependency decision: `zune-jpeg 0.5.15` is used with
  `default-features = false` and `features = ["std"]` to avoid native SIMD
  feature paths in this first safe decoder slice. Its crate license is
  `MIT OR Apache-2.0 OR Zlib`.
- Added a generated `dct-image.pdf` fixture and native backend smoke coverage
  for JPEG-backed Image XObjects.
- Validation so far: `cargo fmt --check`, `git diff --check`, `cargo check`,
  `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`.
- PDFium/native comparison for `dct-image.pdf` at `max-edge 120`: dimensions
  `120x120`, changed pixels `0`, MAE `0.000`, max channel delta `0`,
  native non-white pixels `6400`.
- Left filter chains, CCITT/JPX policy, PNG predictors, and broader JPEG color
  cases for follow-up slices inside this milestone.
