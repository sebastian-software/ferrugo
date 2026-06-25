# 0123: Spreadsheet Grid And Dense Table Fidelity

Status: todo
Phase: 22
Size: medium
Depends on: 0122

## Goal

Make native thumbnails reliable for spreadsheet exports, dense reports, and
table-heavy documents with many thin strokes and small glyphs.

## Scope

- Add spreadsheet-export fixtures with frozen headers, dense grids, and totals.
- Improve hairline stroke consistency for table borders.
- Track small-text readability and clipping behavior.
- Measure rendering cost for pages with many repeated line segments.

## Non-Goals

- Parse spreadsheet files.
- Reconstruct table structure.
- Optimize semantic extraction of cells.

## Deliverables

- Spreadsheet and dense-table corpus fixtures.
- Visual-diff report for grid and small-text fidelity.
- Benchmark evidence for repeated stroke workloads.

## Acceptance Criteria

- Table borders remain visible and stable at thumbnail sizes.
- Dense text does not disappear because of avoidable transform or clipping
  errors.
- Vector-heavy table pages stay within configured item and time budgets.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run spreadsheet corpus comparisons.
- Run vector stress benchmark for dense tables.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
