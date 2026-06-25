# 0129: Mobile Scan And Camera PDF Robustness

Status: todo
Phase: 23
Size: medium
Depends on: 0128

## Goal

Improve robustness for PDFs produced by mobile scanners and camera apps, where
large images, rotation metadata, OCR layers, and compression choices vary.

## Scope

- Add fixtures for rotated scans, cropped images, OCR overlays, and mixed
  compression.
- Verify large-image downsampling and memory behavior.
- Ensure page rotation and crop boxes match expected visual orientation.
- Track unsupported mobile-app image filters explicitly.

## Non-Goals

- Deskew images.
- Run OCR or improve OCR quality.
- Perform document cleanup or enhancement.

## Deliverables

- Mobile scan fixture family.
- Large-image memory and downsampling report.
- Unsupported filter backlog for scanner-produced PDFs.

## Acceptance Criteria

- Common mobile scan PDFs render natively within memory budgets.
- OCR text layers do not visually leak when intended to be invisible.
- Rotation and crop behavior is stable across scanner variants.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run mobile-scan visual comparisons.
- Run large-image memory benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
