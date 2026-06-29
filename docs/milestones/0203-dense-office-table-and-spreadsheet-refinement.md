# 0203: Dense Office Table And Spreadsheet Refinement

Status: done
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

- Produced `docs/reports/dense-office-table-spreadsheet-2026-06-29.md`.
- Reused the existing focused spreadsheet and layout-stress manifests as the
  0203 regression corpus.
- Spreadsheet support gate: 7/7 native, 0 fallback, 0 errors.
- Layout-stress support gate: 7/7 native, 0 fallback, 0 errors.
- Spreadsheet benchmark: 7/7 native, 0 budget failures.
- Layout-stress benchmark: 7/7 native, 0 budget failures.
- Poppler visual review remains a fidelity backlog signal: spreadsheet review
  found 1 accepted drift, 2 blockers, and 4 Poppler reference timeouts; layout
  review found 4 blockers and 3 Poppler reference timeouts.
- Runtime PDFium remains excluded from the supported path.
- Validation:
  - `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --fail-on-fallback --max-edge 160 --output target/dense-office-0203-spreadsheet-support.json`
  - `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --fail-on-fallback --max-edge 160 --output target/dense-office-0203-layout-support.json`
  - `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/dense-office-0203-spreadsheet-benchmark.json`
  - `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/dense-office-0203-layout-benchmark.json`
  - `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/dense-office-0203-spreadsheet-poppler.json`
  - `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/dense-office-0203-layout-poppler.json`
  - `cargo check --workspace --no-default-features`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
