# 0203: Dense Office Table And Spreadsheet Refinement

Status: todo
Phase: 38
Size: medium
Depends on: 0202

## Goal

Raise Rust-native fidelity for dense tables and spreadsheet exports where thin
rules, clipped text, repeated fills, and compact numeric columns expose typical
business-document defects.

## Scope

- Expand fixtures for spreadsheet grids, invoices, financial tables, and dense
  report appendices.
- Validate hairline strokes, clipping, alternating fills, merged cells, rotated
  headers, and repeated XObjects.
- Track visual drift and raster memory usage for large grid pages.
- Add reduced reproductions for top table-layout failures from the scorecard.

## Non-Goals

- Parse spreadsheet source files.
- Infer table semantics beyond rendered PDF output.
- Optimize unrelated image-heavy pages.

## Deliverables

- Dense table and spreadsheet regression corpus.
- Visual drift report for grid-heavy documents.
- Memory and raster hot-path budget update.

## Acceptance Criteria

- Common dense table documents render without missing rules or obvious clipping
  regressions.
- Large grid pages stay within configured memory and time budgets.
- New failures are typed and tied to feature-specific follow-ups.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run dense table corpus visual comparisons.
- Run raster memory benchmark for grid-heavy pages.
- Run supported corpus fallback scan.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
