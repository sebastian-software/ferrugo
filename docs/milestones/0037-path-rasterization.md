# 0037: Path Rasterization

Status: todo
Phase: 3
Size: medium
Depends on: 0036

## Goal

Rasterize basic filled and stroked paths into RGBA thumbnails.

## Scope

- Flatten line and curve path segments.
- Fill nonzero and even-odd paths.
- Stroke simple paths with width and joins sufficient for generated fixtures.
- Add antialiasing strategy for thumbnail output.

## Non-Goals

- Gradients, patterns, transparency groups, or blend modes.
- Perfect PDFium parity for all stroke joins.
- SIMD optimization.

## Deliverables

- Path rasterizer.
- Pixel tests for generated vector fixtures.
- Tolerance policy for path rendering differences.

## Acceptance Criteria

- Simple vector PDFs render through the Rust backend to non-empty RGBA output.
- Pixel comparisons against PDFium pass within documented tolerance.
- Path complexity limits prevent excessive memory and CPU use.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for vector fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
