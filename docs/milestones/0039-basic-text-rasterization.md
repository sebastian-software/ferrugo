# 0039: Basic Text Rasterization

Status: todo
Phase: 3
Size: medium
Depends on: 0038

## Goal

Render enough text for common generated and office-like thumbnails to be
recognizable.

## Scope

- Choose and document the first font rendering dependency or internal strategy.
- Render simple embedded or base fonts used by the fixture set.
- Apply text matrix, font size, and fill color.
- Add reduced fixtures for browser-generated and office-like text PDFs.

## Non-Goals

- Full shaping.
- Full CMap and CID-keyed font coverage.
- Text extraction as a stable API.

## Deliverables

- Basic glyph rasterization path.
- Text fixture pixel comparisons.
- Documentation of unsupported font cases.

## Acceptance Criteria

- Generated text fixtures render visibly through the Rust backend.
- Common simple office/browser text PDFs are recognizable at thumbnail size.
- Unsupported font cases fail with typed errors or visible fallback behavior
  that is documented.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for text fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
