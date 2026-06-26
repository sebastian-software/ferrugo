# 0155: Incremental Loading Interactive Preview

Status: done
Phase: 28
Size: medium
Depends on: 0154

## Goal

Support fast first-page and partial-document preview behavior for large PDFs
without reintroducing PDFium or loading the entire document eagerly.

## Scope

- Audit parse and render paths for first-page latency.
- Reuse linearization, xref, and page scheduling work for preview APIs.
- Add cancellation and partial-result behavior for long renders.
- Measure memory and latency on large documents.

## Non-Goals

- Build a full viewer UI.
- Stream from arbitrary remote protocols in this milestone.
- Sacrifice deterministic output for lower latency.

## Deliverables

- Incremental preview API or internal boundary.
- First-page latency report.
- Cancellation and partial-render tests.

## Acceptance Criteria

- First-page preview does not require full-document rendering for supported
  inputs.
- Cancellation releases work promptly and safely.
- Memory use remains bounded for large documents.

## Validation

- Run first-page latency benchmark.
- Run cancellation tests.
- Run native-only supported corpus gate.
- Run large-document memory profile.

## Completion Notes

Completed on 2026-06-26.

- Added an explicit `NativeBackend::render_first_page_preview` boundary that
  reports whether page-zero rendering used the linearized first-page loader or
  full-document fallback.
- Added `NativeBackend::render_preview_pages_partial` for backend-owned partial
  preview rendering with page-level outcomes, cooperative cancellation, and
  backend-specific render limits.
- Added focused tests for linearized first-page preview, malformed linearized
  fallback, partial page results, and pre-scheduling cancellation.
- Added `fixtures/incremental-preview-manifest.tsv` to group existing
  linearized, page-targeted, multi-page, and longform preview fixtures.
- Native supported gate is green at 5/5 rendered, 0 fallbacks, and 0 errors.
  Default and low-memory benchmark gates report 0 budget failures.
- Report: `docs/reports/incremental-preview-boundary-2026-06-26.md`.
