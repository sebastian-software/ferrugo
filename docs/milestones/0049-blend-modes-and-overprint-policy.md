# 0049: Blend Modes And Overprint Policy

Status: in-progress
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

In progress:

- First implementation slice adds direct `/ExtGState` resource parsing for
  `/BM` values `Normal`, `Compatible`, `Multiply`, and `Screen`.
- Path display-list execution applies `/gs` graphics-state resources and stores
  the active blend mode in each path item.
- Path rasterization applies `Normal`, `Multiply`, and `Screen` with
  allocation-free channel math inside the pixel loop.
- Enabled `/OP true` and `/op true` overprint policies fail with typed
  unsupported errors instead of silently rendering incorrect thumbnails.
- Native rendering now resolves page-level `/Resources /ExtGState` dictionaries
  for the path pass.
- Current validation:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check`
  - `cargo test --quiet`
  - `cargo test -p pdfrust-render ext_graphics_state -- --nocapture`
  - `cargo test -p pdfrust-render blend_source_with_backdrop -- --nocapture`
  - `cargo test -p pdfrust-render display_list_should_apply_external_graphics_state_blend_mode -- --nocapture`
  - `cargo clippy --all-targets --all-features -- -D warnings`
