# 0130: Legal Contract And Redaction Document Coverage

Status: done
Phase: 23
Size: medium
Depends on: 0129

## Goal

Validate native rendering for contracts, legal filings, redacted PDFs, and
signature-heavy documents that mix text, annotations, stamps, and scans.

## Scope

- Add fixtures for contracts, filing-style pages, visible redactions, and
  signature blocks.
- Cover stamps, highlights, comments, typed fields, and scanned attachments.
- Track annotation and form fallback outcomes for legal-document subtypes.
- Verify that redaction visuals render as page content or appearances.

## Non-Goals

- Validate legal signatures.
- Determine whether redactions are semantically secure.
- Extract contract clauses or legal metadata.

## Deliverables

- Legal-document fixture family.
- Annotation/form coverage report for contracts and filings.
- Redaction visual-rendering policy notes.

## Acceptance Criteria

- Visible redactions, stamps, and signatures appear in native thumbnails.
- Missing appearance cases are typed, synthesized, or documented.
- Legal-document blockers are traceable to concrete renderer features.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run legal-document visual comparisons.
- Run annotation and form fixture checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated legal fixtures covering contract signature/stamp
  blocks, court filing stamp/comment/highlight visuals, visible redaction
  rectangles, and a two-page scanned attachment packet.
- Added `fixtures/legal-document-manifest.tsv` with 13 focused rows across
  `contract`, `filing`, `redaction`, `scanned-attachment`, and
  `missing-appearance` families.
- Added native regression coverage for legal fixture rendering, visible black
  redaction rectangles, and parallel sampling of scanned legal attachment
  pages.
- Native fallback gate: 13/13 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 13/13 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle at strict default thresholds: 3 exact matches,
  3 accepted drift, 7 blockers, 0 native render errors, 0 PDFium render
  errors. The blockers are visual-fidelity differences in signatures, form
  synthesis, text/stamp rasterization, scans, and redaction rectangle edges.
- Report: `docs/reports/legal-document-coverage-2026-06-25.md`.
