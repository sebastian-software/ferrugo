# 0034: Image XObject Decoding And Placement

Status: todo
Phase: 2
Size: medium
Depends on: 0033

## Goal

Decode and place common image XObjects in the display list.

## Scope

- Resolve image XObject resources.
- Support DeviceRGB and DeviceGray image metadata.
- Decode Flate-backed images.
- Add DCT/JPEG decoding strategy or a narrow implementation behind a safe API.
- Store positioned image items in the display list.

## Non-Goals

- JPX/JPEG 2000, CCITT, or JBIG2.
- Full color management.
- Image interpolation tuning.

## Deliverables

- Image XObject resolver.
- Image display-list items.
- Tests for generated image fixtures.

## Acceptance Criteria

- Generated PDFs with embedded RGB images produce image display-list items.
- Unsupported image filters and color spaces return typed errors.
- Image byte ownership is explicit and avoids redundant copies where practical.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare image placement metadata against PDFium-rendered fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
