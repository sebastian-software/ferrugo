# 0088: Image Mask Stencil And Bitmap Edge Cases

Status: todo
Phase: 15
Size: medium
Depends on: 0087

## Goal

Handle common image mask and stencil patterns used in logos, signatures, icons,
and scanned overlays.

## Scope

- Implement ImageMask handling with current fill color.
- Cover decode inversion, one-bit masks, and stencil placement transforms.
- Add fixtures for signatures, monochrome icons, and masked logos.
- Preserve image memory budgets for large bitmap inputs.

## Non-Goals

- Add new compressed image codecs in this milestone.
- Support arbitrary color-managed proofing.
- Store expanded masks longer than needed for rendering.

## Deliverables

- ImageMask rendering support.
- Mask-focused fixture set.
- Memory notes for one-bit and expanded mask paths.

## Acceptance Criteria

- Stencil masks render with correct color and decode direction.
- Mask expansion is bounded and page-local unless explicitly cached.
- Visual comparisons match PDFium for typical masked bitmap documents.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run image-mask visual comparisons.
- Run large-mask memory checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
