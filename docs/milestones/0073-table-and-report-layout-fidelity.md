# 0073: Table And Report Layout Fidelity

Status: done
Phase: 12
Size: medium
Depends on: 0072

## Goal

Improve native fidelity for reports, invoices, statements, and table-heavy PDFs.

## Scope

- Add fixtures with dense ruled tables, small text, logos, and repeated headers.
- Tune stroke, text, and image placement at thumbnail sizes.
- Track legibility and missing-element regressions separately from pixel deltas.
- Add multi-page report comparison coverage.

## Non-Goals

- Extract table structure.
- Infer semantic document fields.
- Optimize for arbitrary zoom levels before thumbnail parity is stable.

## Deliverables

- Report and table fixture suite.
- Native fixes for common small-text and ruling-line issues.
- Multi-page comparison output for representative reports.

## Acceptance Criteria

- Typical reports render with visible text, table lines, and logos.
- Multi-page documents preserve stable dimensions and page ordering.
- Remaining differences have clear renderer feature labels.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run report corpus comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `test: add multi-page report coverage` implementation and
the `docs: complete report layout coverage` report update.

- Added `fixtures/generated/multi-page-report.pdf`, a generated two-page report
  with repeated header styling, a logo marker, table ruling lines, and text
  cells.
- Added native-backend render coverage for the first page.
- Added native-backend metadata coverage for two-page ordering and dimensions.
- Verified PDFium/native metadata parity and documented render fidelity limits
  in `docs/reports/report-layout-coverage-2026-06-24.md`.
