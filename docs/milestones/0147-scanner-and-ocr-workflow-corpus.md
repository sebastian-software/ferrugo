# 0147: Scanner And OCR Workflow Corpus

Status: done
Phase: 27
Size: medium
Depends on: 0146

## Goal

Cover scanned and OCR-layer PDFs that appear in document management, mailroom,
mobile capture, and archival workflows.

## Scope

- Add fixtures for image-only scans, OCR text overlays, rotated scans, skewed
  pages, large images, and mixed scan-plus-form documents.
- Track memory and decode costs for high-resolution pages.
- Verify invisible text and image layers do not corrupt visual output.
- Add diagnostics for extreme image dimensions or decode budgets.

## Non-Goals

- Run OCR.
- Improve source image quality.
- Support unlimited image dimensions without budgets.

## Deliverables

- Scanner/OCR corpus entries.
- Memory and decode budget report.
- Follow-up backlog for image-heavy rendering gaps.

## Acceptance Criteria

- Native renderer handles common scan pages without PDFium fallback.
- OCR text layers are preserved or ignored according to documented policy.
- Large-image memory behavior stays within configured budgets.

## Validation

- Run scan-family visual comparison.
- Run memory profile for high-resolution fixtures.
- Run native-only supported corpus gate.
- Run malformed image budget tests.

## Completion Notes

- Added generated scanner/OCR fixtures for skewed mailroom scans, large
  compressed scan images, and scan-plus-form OCR overlays.
- Added `fixtures/scanner-ocr-workflow-manifest.tsv` to separate supported
  scanner/OCR workflow rows from the unsupported image-codec backlog.
- Native supported gate passed: 10/10 supported scanner/OCR fixtures rendered
  natively, with 0 fallbacks and 0 errors.
- Unsupported filter backlog remains explicit: 3/3 rows require
  `image.filter` fallback.
- Native benchmark passed with 0 budget failures; the large-image fixture
  rendered in 33.527 ms at `max-edge 160`.
- Visual oracle classified 6/10 supported rows as fidelity blockers, routing to
  scan resampling, page geometry/skew parity, and overlay composition.
- Existing malformed/large image budget tests passed for adversarial huge image
  dimensions and image resource byte budgets.
- Report: `docs/reports/scanner-ocr-workflow-corpus-2026-06-26.md`.
