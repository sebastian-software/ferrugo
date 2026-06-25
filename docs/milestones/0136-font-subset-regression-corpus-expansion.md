# 0136: Font Subset Regression Corpus Expansion

Status: done
Phase: 25
Size: medium
Depends on: 0135

## Goal

Expand regression coverage for embedded font subsets that frequently appear in
office, browser, report, and publishing exports.

## Scope

- Add reduced fixtures for TrueType, CFF, CID, Type3, and missing-font cases.
- Track glyph mapping, widths, encodings, and fallback behavior by font class.
- Reduce real-world font failures into minimal shareable PDFs.
- Measure glyph cache behavior across repeated subset fonts.

## Non-Goals

- Build a general-purpose font engine beyond PDF needs.
- Download replacement fonts.
- Treat every script and shaping case as complete.

## Deliverables

- Font subset regression corpus.
- Font-class support matrix update.
- Glyph cache and mapping report.

## Acceptance Criteria

- Common subset font failures have dedicated regression fixtures.
- Font fallback and unsupported cases are typed and documented.
- Repeated subset fonts do not cause unbounded cache growth.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run font subset corpus comparisons.
- Run glyph cache benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Added five generated reduced fixtures for subset TrueType, CFF, Type0 CID,
  Type3, and missing-font cases.
- Added `fixtures/font-subset-manifest.tsv` and registered the same fixtures in
  the main corpus manifest under `office-export`.
- Added native backend regression coverage proving the fixtures render visibly
  without fallback.
- Documented the font subset support matrix and measured benchmark results in
  `docs/reports/font-subset-regression-2026-06-26.md`.
- Validation passed: `cargo fmt --check`, focused native test, font subset
  fallback summary, font subset benchmark, `cargo check --workspace`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace`, and `cargo test --workspace --no-default-features`.
