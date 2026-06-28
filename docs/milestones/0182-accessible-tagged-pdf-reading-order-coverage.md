# 0182: Accessible Tagged PDF Reading Order Coverage

Status: in-progress
Phase: 34
Size: medium
Depends on: 0181

## Goal

Use tagged PDF structure to validate visual integrity and expose enough reading
order signals for searchable, accessible, and reviewable typical documents.

## Scope

- Parse structure tree relationships needed for page content association.
- Detect reading-order mismatches that indicate hidden or misplaced visual text.
- Add fixtures for tagged reports, forms, invoices, and generated office PDFs.
- Document the boundary between visual rendering, text extraction, and
  accessibility metadata.

## Non-Goals

- Implement a full accessibility API.
- Repair incorrect producer tags.
- Change visual output solely to match logical order.

## Deliverables

- Tagged PDF coverage report.
- Structure tree parsing or classification improvements.
- Fixtures covering common tagged producer output.

## Acceptance Criteria

- Tagged PDFs do not regress visual rendering.
- Reading-order signals are classified without excessive allocations.
- Unsupported accessibility structures are documented and typed.

## Validation

- Run native-only `cargo test`.
- Run tagged PDF fixture rendering.
- Run text and metadata classification tests.
- Run visual comparison for tagged fixtures.

## Progress Notes

Native slice committed toward 0182 on 2026-06-28.

- Added bounded reading-order metadata signals:
  `marked_content_reference_count`, `page_content_reference_count`,
  `alt_text_count`, and `reading_order_warning_count`.
- Added `tagged-invoice-reading-order.pdf` for typical invoice/header/table
  reading-order associations.
- Added `tagged-reading-order-missing-page-context.pdf` as a warning-boundary
  fixture for MCID references without page context.
- Updated `fixtures/tagged-pdf-visual-manifest.tsv` and
  `fixtures/corpus-manifest.tsv`.
- Added `docs/reports/tagged-reading-order-coverage-2026-06-28.md`.
- Native gates are green: 7 tagged fixtures render natively with 0 fallback,
  0 errors, and 0 benchmark budget failures.
- PDFium visual-diff was attempted, but the local `libpdfium.dylib` oracle was
  not available.
- Added PDFium-free `visual-diff-poppler` maintainer tooling. The tagged
  Poppler oracle run completed with 0 native errors and 0 reference errors, but
  all 7 tagged fixtures remain strict visual blockers. 0182 remains in progress
  for fidelity work, not for oracle availability.
- Refined standard-base-font fallback masks so Helvetica-style tagged fixtures
  paint less heavily. This reduces several Poppler diff metrics but does not
  yet clear the tagged visual gate.
- Aligned `visual-diff-poppler` to render Poppler references at the native
  target dimensions when native rendering succeeds. The two former 1px
  dimension mismatches now produce comparable Poppler metrics, while all 7
  fixtures remain strict visual blockers.

## Completion Notes

Empty until done.
