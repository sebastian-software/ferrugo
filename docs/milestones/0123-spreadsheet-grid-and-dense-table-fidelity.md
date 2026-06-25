# 0123: Spreadsheet Grid And Dense Table Fidelity

Status: done
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

Completed on 2026-06-25.

- Added synthetic spreadsheet fixtures for frozen headers, dense numeric grids,
  clipped cells, and repeated-stroke stress grids.
- Added `fixtures/spreadsheet-grid-manifest.tsv` for table-heavy subtype gates.
- Added native regression coverage that checks dimensions and dense visible
  grid/text pixels.
- Native spreadsheet gate renders 7/7 manifest rows without fallback or errors.
- Native benchmark has 0 budget failures; the stress-grid family stayed below
  the 1000 ms budget with max 191.991 ms.
- PDFium visual oracle reports 7 fidelity blockers under strict thresholds,
  with no native or PDFium render errors.
