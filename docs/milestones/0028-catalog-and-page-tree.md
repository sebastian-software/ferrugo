# 0028: Catalog And Page Tree

Status: done
Phase: 1
Size: medium
Depends on: 0027

## Goal

Resolve the document catalog and page tree into safe page metadata.

## Scope

- Locate the catalog from the trailer root.
- Traverse page tree nodes.
- Inherit page boxes and resource references.
- Expose page count and page size through Rust-native document APIs.

## Non-Goals

- Interpret content streams.
- Render pages.
- Resolve every metadata field.

## Deliverables

- Catalog and page tree resolver.
- Page metadata structs.
- Tests for inherited boxes, malformed trees, and cycle detection.

## Acceptance Criteria

- The Rust-native loader can report page count and first-page size for generated
  fixtures.
- Page tree cycles and missing required fields return typed errors.
- The public thumbnail facade remains backend-neutral.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare page count and page size against PDFium for generated fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-24.

- Added `PageTree`, `PageMetadata`, `PageBox`, and `PageSize`.
- Added `ClassicDocument::page_tree` and `ModernDocument::page_tree`.
- Resolved trailer `/Root`, catalog `/Pages`, page tree `Kids`, and page leaf
  metadata.
- Added inherited `MediaBox`, `CropBox`, and resource-reference handling.
- Added typed errors for missing objects, malformed page-tree fields, page-tree
  cycles, and invalid page boxes.
- Added tests for classic page metadata inheritance, modern compressed page
  metadata, missing media boxes, and page-tree cycles.
- Content stream interpretation and rendering remain later milestones.
