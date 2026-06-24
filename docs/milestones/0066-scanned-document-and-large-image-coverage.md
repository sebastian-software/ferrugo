# 0066: Scanned Document And Large Image Coverage

Status: done
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

- Added generated `fixtures/generated/scanned-page.pdf`, a page-sized
  DeviceGray image placement fixture.
- Added `scanned-page.pdf` to the `scan` corpus family in
  `fixtures/corpus-manifest.tsv` with source, license, page-count, feature, and
  note metadata.
- Added native backend smoke coverage for the scan-like page fixture.
- Added `docs/reports/scanned-document-coverage-2026-06-24.md`.
- Scan corpus summary at `--max-edge 120` reported 7 total fixtures, 7 native
  renders, 1.000 native pass rate, 0 fallbacks, and 0 errors.
- PDFium differential smoke at `--max-edge 200` rendered all 7 scan fixtures
  successfully through native and direct PDFium with matching PNG dimensions.
- Memory diagnostics for `scanned-page.pdf` were captured through
  `compare-metadata`; native page metadata matched PDFium and reported current
  page, image, font, CMap, text-run, and display-item budgets.
- Validation: `cargo fmt --check`, `cargo check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test --quiet`, manifest/PDF set comparison, scan corpus summary,
  PDFium differential smoke, and scan metadata/memory diagnostics.
