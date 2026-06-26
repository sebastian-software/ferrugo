# 0162: PDF 2.0 Compatibility Boundary Gate

Status: done
Phase: 30
Size: medium
Depends on: 0161

## Goal

Define and validate the Rust renderer boundary for PDF 2.0 documents so common
producer output is handled explicitly instead of failing through ambiguous parse
or render errors.

## Scope

- Inventory PDF 2.0 features that appear in typical office, browser, form, and
  scan workflows.
- Add fixtures for accepted PDF 2.0 syntax that maps cleanly to existing
  renderer behavior.
- Add typed unsupported errors for PDF 2.0 features that are outside the current
  native renderer boundary.
- Document the compatibility policy in native backend docs.

## Non-Goals

- Implement every PDF 2.0 feature.
- Add runtime PDFium fallback for unsupported PDF 2.0 files.
- Change behavior for PDF 1.x files without fixture coverage.

## Deliverables

- PDF 2.0 compatibility policy.
- Native parser and renderer fixtures for accepted cases.
- Typed unsupported coverage for rejected cases.

## Acceptance Criteria

- Common accepted PDF 2.0 files render or classify deterministically.
- Unsupported PDF 2.0 features return typed errors with actionable names.
- Compatibility docs avoid overstating support.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run PDF 2.0 fixture corpus.
- Run fallback summary for affected document families.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added the first PDF 2.0 fixture corpus in
  `fixtures/pdf20-compatibility-manifest.tsv`.
- Added generated fixtures for accepted `%PDF-2.0` / catalog `/Version /2.0`
  rendering, accepted associated-file metadata, and typed unsupported black
  point compensation.
- Added `docs/policies/pdf-2-0-compatibility.md` and
  `docs/reports/pdf-2-0-compatibility-boundary-2026-06-26.md`.
- Native renderer maps `/UseBlackPtComp true` to typed unsupported bucket
  `graphics.color-management`.
- Native-only check/test/clippy, focused PDF 2.0 tests, PDF 2.0 fallback
  summaries, and affected supported-family fallback gate passed.
