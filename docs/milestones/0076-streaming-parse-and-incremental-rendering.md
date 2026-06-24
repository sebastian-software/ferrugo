# 0076: Streaming Parse And Incremental Rendering

Status: todo
Phase: 12
Size: medium
Depends on: 0075

## Goal

Reduce peak memory by parsing and rendering only the data needed for requested
pages where the file structure allows it.

## Scope

- Audit current full-document loading behavior.
- Add page-targeted object and stream access for render paths.
- Avoid retaining decoded streams after page rendering unless cached by policy.
- Preserve deterministic errors for malformed cross-reference structures.

## Non-Goals

- True network streaming.
- Random access without a seekable input source.
- Rewrite all object storage in one step.

## Deliverables

- Page-targeted loading path.
- Memory comparison notes before and after the change.
- Tests proving unrelated pages are not decoded for single-page thumbnails.

## Acceptance Criteria

- Single-page rendering avoids unnecessary stream decode work.
- Peak memory drops on multi-page image-heavy fixtures.
- Object lifetimes remain explicit and borrow-friendly.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run multi-page memory measurements.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
