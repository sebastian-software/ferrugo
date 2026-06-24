# 0066: Scanned Document And Large Image Coverage

Status: todo
Phase: 10
Size: medium
Depends on: 0065

## Goal

Make image-heavy and scanned PDFs render natively within bounded memory.

## Scope

- Cover page-sized images, mixed DPI pages, masks, rotations, and decode arrays.
- Add downsampling paths that avoid decoding more pixels than needed.
- Enforce image memory budgets before allocation.
- Compare native output against PDFium for representative scanned documents.

## Non-Goals

- OCR text extraction.
- Color-managed archival rendering.
- Unlimited high-resolution raster export.

## Deliverables

- Scanned-document fixture coverage.
- Bounded image decode and scaling path.
- Memory diagnostics for large image pages.

## Acceptance Criteria

- Large scanned pages render without unbounded allocation.
- Thumbnail rendering avoids full-resolution intermediate buffers where possible.
- Oversized inputs fail with budget errors, not process exhaustion.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run large-image corpus comparisons and memory checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
