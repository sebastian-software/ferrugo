# 0050: Patterns Shadings And Gradients

Status: todo
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

Empty until done.
