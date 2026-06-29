# 0068: Complex Font Subsetting And CID Fonts

Status: done
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

Completed on 2026-06-24.

- Added explicit Type0/CID descendant font metadata decoding, including
  `CIDFontType0`/`CIDFontType2` subtype validation and `/DW` default widths.
- Accepted `Identity-H` composite font encoding when ToUnicode supplies character
  mapping.
- Applied CID `/DW` as fallback text advance width for Type0/CID text runs.
- Added `fixtures/generated/cid-font-text.pdf`, a generated Type0 CID font
  fixture with two-byte character codes, ToUnicode mapping, and `/DW 600`.
- Added render and native backend tests for Type0/CID decoding and the generated
  fixture.
- Recorded fallback and PDFium comparison results in
  `docs/reports/cid-font-coverage-2026-06-24.md`.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo test -p ferrugo-render`
  - `cargo test -p ferrugo-native`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - font-heavy fallback summary through the office-export family
  - native/PDFium render comparison for `cid-font-text.pdf`
