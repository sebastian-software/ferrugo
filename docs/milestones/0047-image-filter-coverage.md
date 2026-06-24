# 0047: Image Filter Coverage

Status: todo
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

Empty until done.
