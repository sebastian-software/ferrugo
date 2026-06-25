# 0113: Embedded Files Portfolio And Attachment Visibility

Status: done
Phase: 20
Size: medium
Depends on: 0112

## Goal

Handle PDFs with embedded files, portfolios, and attachment annotations without
breaking thumbnail rendering.

## Scope

- Parse embedded-file and collection dictionaries needed for classification.
- Render attachment annotations when appearance streams are present.
- Expose portfolio or attachment presence in metadata.
- Add fixtures for email exports, portfolios, and attached source documents.

## Non-Goals

- Extract or open embedded files by default.
- Execute attached content.
- Implement a portfolio browser.

## Deliverables

- Embedded-file metadata classification.
- Attachment appearance rendering coverage.
- Portfolio fixture report.

## Acceptance Criteria

- Portfolio PDFs do not crash or consume unbounded memory.
- Attachment indicators render when visual appearances exist.
- Embedded content remains inert unless a future explicit API requests it.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run portfolio and attachment fixture comparisons.
- Run metadata extraction smoke tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed 2026-06-25.

- Added generated embedded-file, portfolio, and file-attachment annotation
  fixtures.
- Exposed inert presence-only metadata for embedded files, portfolio
  collections, and file-attachment annotations.
- Covered visible file-attachment annotations through the existing appearance
  rendering path.
- Documented the boundary: no extraction, execution, or portfolio UI in the
  native thumbnail renderer.
- Validation evidence is recorded in
  `docs/reports/embedded-files-portfolio-2026-06-25.md`.
