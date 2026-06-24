# 0068: Complex Font Subsetting And CID Fonts

Status: todo
Phase: 11
Size: medium
Depends on: 0067

## Goal

Handle common subset and CID font patterns found in real-world PDFs.

## Scope

- Add Type0 and CIDFont resource handling needed by typical documents.
- Resolve descendant font metrics, widths, and CMaps.
- Keep glyph outline extraction cached and budgeted.
- Add fixtures for subset names, composite fonts, and missing optional metrics.

## Non-Goals

- Implement every historical font technology.
- Build a standalone font engine.
- Support malformed font programs beyond documented recovery policy.

## Deliverables

- CID-aware font resource model.
- Tests for composite font decoding and metrics.
- Corpus gap report for remaining font failures.

## Acceptance Criteria

- Common CID-backed text renders with correct glyph selection.
- Font caches remain bounded across multi-page documents.
- Unsupported font cases return actionable error categories.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run font-heavy corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
