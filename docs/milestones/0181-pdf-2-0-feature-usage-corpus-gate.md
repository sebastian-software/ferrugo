# 0181: PDF 2.0 Feature Usage Corpus Gate

Status: done
Phase: 34
Size: medium
Depends on: 0180

## Goal

Measure which PDF 2.0 features appear in typical documents and decide which
ones must be supported, approximated, or typed as unsupported for the native
renderer.

## Scope

- Expand corpus metadata with PDF version and producer feature flags.
- Detect common PDF 2.0 structures that affect visual output.
- Add typed unsupported reasons for observed features outside the current
  renderer boundary.
- Rank implementation follow-ups by real document frequency and render impact.

## Non-Goals

- Implement complete PDF 2.0 coverage.
- Treat rare archival features as release blockers without corpus evidence.
- Reintroduce PDFium runtime fallback.

## Deliverables

- PDF 2.0 usage report.
- Corpus feature classification updates.
- Ranked follow-up backlog.

## Acceptance Criteria

- PDF 2.0 documents are detected and classified in corpus reports.
- Visual-impacting unsupported features have stable typed reasons.
- The 1.2 roadmap has evidence-backed PDF 2.0 priorities.

## Validation

- Run native-only `cargo test`.
- Run corpus classification across PDF version families.
- Run supported corpus fallback summary.
- Review unsupported feature categories for ambiguity.

## Completion Notes

Completed on 2026-06-28.

- Added `classify-pdf20-usage` to `pdfrust-cli` for privacy-safe PDF 2.0
  corpus classification by version evidence, manifest feature flags,
  visual-impact policy, native render outcome, and ranked follow-ups.
- Added `docs/reports/pdf-2-0-feature-usage-corpus-2026-06-28.md` and
  `docs/backlogs/pdf-2-0-feature-priority-backlog.md`.
- Updated `docs/policies/pdf-2-0-compatibility.md` so the 1.2 roadmap uses the
  classifier-backed backlog for PDF 2.0 prioritization.
- Current generated corpus result: 211 PDFs scanned, 3 PDF 2.0 documents
  detected, 2 native rendered, 1 typed unsupported, 0 errors.
- Supported PDF 2.0 subset gate: 2 total, 2 native rendered, 0 fallback, 0
  errors.
- Full PDF 2.0 classification: 3 total, 2 native rendered, 1 fallback required
  under `graphics.color-management`, 0 errors.
- Validation: `cargo fmt`, focused `cargo test -p pdfrust-cli pdf20_usage`,
  PDF 2.0 usage classifier, supported subset fallback gate, and full PDF 2.0
  fallback classification.
