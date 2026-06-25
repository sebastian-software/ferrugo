# 0147: Scanner And OCR Workflow Corpus

Status: todo
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

Empty until done.
