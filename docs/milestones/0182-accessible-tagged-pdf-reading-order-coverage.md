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
- Added case-sensitive standard-font fallback bitmaps with lowercase x-height
  masks. This improves the text-heavy Poppler metrics, but the strict tagged
  visual gate still has 7 blockers.
- Added coverage-aware antialiasing for standard text fallback rectangles. The
  tagged reading-order warning fixture now classifies as accepted Poppler drift,
  reducing the strict tagged visual blockers from 7 to 6.
- Scaled the standard-base-font fallback cell to better approximate Helvetica
  cap height. This reduces several text-heavy drift metrics, but the strict
  tagged visual gate still has 6 blockers.
- Added deterministic Helvetica advance widths for standard-base-font fallback
  glyph positioning. This reduces text-heavy drift further, especially the
  tagged form fixture p95 delta, while the strict visual gate still has 6
  blockers.
- Completed the printable ASCII Helvetica width table for standard-base-font
  fallback positioning. The tagged reading-order warning fixture remains
  accepted Poppler drift with lower changed ratio and max channel delta, while
  the strict tagged visual gate still has 6 blockers.
- Completed printable ASCII fallback bitmap coverage instead of routing
  punctuation through the unknown-glyph mask. The tagged invoice fixture drift
  improved slightly, while the strict tagged visual gate still has 6 blockers.
- Re-ran the tagged Poppler oracle after the later StandardBase glyph-weight
  tuning from 0183. The strict tagged visual gate now reports 7 total,
  0 exact, 3 accepted drift, and 4 blockers: metadata-baseline,
  tagged-form, and reading-order-warning are accepted, while invoice, office
  alt-text, report, and structure-heavy fixtures remain visual blockers.
- Re-ran the tagged Poppler oracle after the later 0183 narrow hairline
  forward-snap tuning. The strict tagged visual gate now reports 7 total,
  0 exact, 4 accepted drift, and 3 blockers: tagged office alt-text moved to
  accepted drift, while invoice, report, and structure-heavy fixtures remain
  visual blockers.
- Re-ran the tagged Poppler oracle after limiting the later 0183
  forward-fraction snap to vertical hairlines only. The strict tagged visual
  gate returns to 7 total, 0 exact, 3 accepted drift, and 4 blockers, but
  `tagged-report-visual-integrity.pdf` improves to MAE 2.647, p95 7,
  changed ratio 0.105011, max delta 150.
- Re-ran the tagged Poppler oracle after the later 0183 large axis-aligned
  rectangle fill center-sampling change. The strict tagged visual gate improves
  to 7 total, 0 exact, 5 accepted drift, and 2 blockers. The tagged invoice
  and report fixtures moved to accepted drift; `tagged-office-alt-text.pdf` and
  `tagged-structure-heavy-report.pdf` remain blockers.

## Completion Notes

Empty until done.
