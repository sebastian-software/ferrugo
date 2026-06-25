# 0113: Embedded Files Portfolio And Attachment Visibility

Status: in-progress
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

Empty until done.
