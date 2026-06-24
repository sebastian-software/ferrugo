# 0076: Streaming Parse And Incremental Rendering

Status: done
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

Completed with the `test: add page-targeted stream coverage` and
`feat: filter unused page xobjects` implementation commits plus the
`docs: complete streaming parse coverage` report update.

- Added `fixtures/generated/page-targeted-stream.pdf`, a two-page fixture that
  combines a valid page 0, an unused malformed page-0 Image XObject, and a
  malformed page-1 content stream.
- Added native-backend tests proving page 0 renders without decoding unrelated
  page streams or unused page XObjects, while page 1 fails when explicitly
  requested.
- Filtered page-level Image/Form XObject resource decoding to names actually
  invoked by `Do` operators in optional-content-filtered page content.
- Recorded corpus summary and page-0 memory measurements in
  `docs/reports/streaming-parse-coverage-2026-06-24.md`.
