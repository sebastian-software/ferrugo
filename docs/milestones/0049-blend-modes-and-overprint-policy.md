# 0049: Blend Modes And Overprint Policy

Status: todo
Phase: 6
Size: medium
Depends on: 0048

## Goal

Implement the highest-value blend modes and define a practical overprint policy
for thumbnails.

## Scope

- Support normal, multiply, screen, and other corpus-driven blend modes.
- Keep blend operations branch-light and allocation-free per pixel.
- Define whether overprint is ignored, approximated, or unsupported.
- Add fixtures for blend-heavy browser and design-tool exports.

## Non-Goals

- Full print-production overprint fidelity.
- DeviceN spot color parity.
- Color-managed blending.

## Deliverables

- Blend-mode implementation for prioritized modes.
- Overprint policy documentation.
- Differential tests for supported blend modes.

## Acceptance Criteria

- Supported blend fixtures are visually close enough for thumbnails.
- Unsupported blend or overprint cases are explicit and stable.
- Blend operations do not allocate in the inner pixel loop.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for blend fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
