# 0050: Patterns Shadings And Gradients

Status: in-progress
Phase: 6
Size: medium
Depends on: 0049

## Goal

Render common pattern and shading fills that appear in reports, slides, and
browser-generated PDFs.

## Scope

- Support simple tiling patterns with bounded repeat counts.
- Support axial and radial shadings at thumbnail resolution.
- Cache sampled shading results where it reduces repeated work.
- Return typed unsupported errors for mesh shadings and complex patterns.

## Non-Goals

- Printer-grade gradient precision.
- Full mesh shading support.
- Infinite pattern recursion.

## Deliverables

- Pattern and shading render paths.
- Fixtures for tiling patterns and gradients.
- Sampling and recursion-limit tests.

## Acceptance Criteria

- Common gradient and simple pattern PDFs render recognizably.
- Pattern recursion and tile expansion are bounded.
- Unsupported shading types do not break the page render.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for shading fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

In progress:

- First implementation slice adds direct `/Resources /Shading` parsing for
  `/ShadingType 2` axial shadings with `DeviceRGB` or `DeviceGray` Type 2
  sampled functions.
- Content-stream `sh` operators now produce shading display-list items when the
  named resource is supported, and unsupported shading types fail with typed
  errors.
- Axial shading rasterization projects device pixels onto the gradient axis and
  samples colors without per-pixel heap allocation.
- Native rendering now resolves page-level `/Shading` dictionaries for the path
  and form scan passes.
- Current validation:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check`
  - `cargo test --quiet`
  - `cargo test -p pdfrust-render shading -- --nocapture`
  - `cargo clippy --all-targets --all-features -- -D warnings`
