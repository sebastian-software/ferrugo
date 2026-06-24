# 0085: Page Geometry Boxes Rotation And User Units

Status: todo
Phase: 15
Size: medium
Depends on: 0084

## Goal

Match common PDF page geometry behavior across media boxes, crop boxes,
rotation, and user units.

## Scope

- Implement effective page box selection for rendering.
- Handle page rotation and non-default user units consistently.
- Add fixtures for rotated office exports, cropped scans, and unusual page
  sizes.
- Keep raster dimensions bounded by thumbnail options and memory policy.

## Non-Goals

- Add print imposition support.
- Render outside the selected page box.
- Accept unbounded page dimensions.

## Deliverables

- Page geometry implementation updates.
- Geometry fixture set.
- Native/PDFium comparison report.

## Acceptance Criteria

- Rendered thumbnail dimensions match the selected page geometry.
- Rotation and crop behavior match PDFium for typical documents.
- Path, image, and text placement remain stable after geometry transforms.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run geometry-focused corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
