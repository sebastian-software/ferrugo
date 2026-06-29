# Dense Office Table And Spreadsheet Refinement

Date: 2026-06-29
Milestone: 0203

## Decision

The dense table and spreadsheet slice is native-runtime ready, but not visually
parity-complete. Focused support and benchmark gates pass with zero fallbacks,
errors, or budget failures. Independent Poppler review still shows dense-grid
and layout drift, so this milestone records the baseline and keeps visual
fidelity follow-ups visible instead of hiding them behind a PDFium runtime
fallback.

## Corpus

0203 uses two existing focused manifests:

- `fixtures/spreadsheet-grid-manifest.tsv`
- `fixtures/layout-stress-manifest.tsv`

These cover frozen headers, dense numeric grids, clipped cells, repeated vector
grid stress, dense business tables, two-column report layouts, footnotes, and
page furniture.

## Native Support Gates

Spreadsheet support:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 0 | 0 |

Layout-stress support:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 0 | 0 |

## Benchmark And Memory Budget

| Gate | Total | Native rendered | Fallbacks | Errors | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| Spreadsheet grid | 7 | 7 | 0 | 0 | 0 |
| Layout stress | 7 | 7 | 0 | 0 | 0 |

Existing native tests also keep grid-heavy pages from silently losing dense
content:

- `native_backend_should_render_generated_spreadsheet_grid_fixtures`
- `native_low_memory_profile_should_render_common_thumbnail_fixtures`

## Poppler Visual Review

Poppler is used here only as an independent review oracle. It is not part of
the supported runtime path.

| Gate | Total | Accepted drift | Blockers | Native errors | Reference errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| Spreadsheet grid | 7 | 1 | 2 | 0 | 4 |
| Layout stress | 7 | 0 | 4 | 0 | 3 |

Spreadsheet blockers:

| Fixture | Family | Subsystem | Main metrics |
| --- | --- | --- | --- |
| `spreadsheet-frozen-header.pdf` | `frozen-header` | `rendering-core` | MAE 26.454, p95 221, changed 0.182 |
| `spreadsheet-vector-stress-grid.pdf` | `stress-grid` | `vector-graphics` | MAE 47.737, p95 186, changed 0.447 |

Layout blockers:

| Fixture | Family | Subsystem | Main metrics |
| --- | --- | --- | --- |
| `office-report-header-footer-link.pdf` | `page-furniture` | `rendering-core` | MAE 9.315, p95 32, changed 0.234 |
| `reference-footnote-layout.pdf` | `footnotes` | `rendering-core` | MAE 12.056, p95 108, changed 0.135 |
| `scientific-two-column-paper.pdf` | `two-column` | `rendering-core` | MAE 12.576, p95 104, changed 0.137 |
| `spreadsheet-dense-numeric-grid.pdf` | `spreadsheet-grid` | `rendering-core` | MAE 23.690, p95 113, changed 0.309 |

The reference errors are Poppler timeouts on dense fixtures in the local review
run, not native renderer errors. They are kept visible so future review tooling
can distinguish native regressions from reference-tool limits.

## Follow-Ups

1. Reduce dense-grid and vector-stress drift around repeated thin strokes.
2. Tighten small text and table-rule alignment for spreadsheet/report layouts.
3. Keep clipping regressions covered by the spreadsheet and layout-stress
   manifests.
4. Revisit Poppler timeout handling for dense reference pages before treating
   reference errors as release blockers.

## Validation

Commands run:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --fail-on-fallback --max-edge 160 --output target/dense-office-0203-spreadsheet-support.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --fail-on-fallback --max-edge 160 --output target/dense-office-0203-layout-support.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/dense-office-0203-spreadsheet-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/dense-office-0203-layout-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/dense-office-0203-spreadsheet-poppler.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/dense-office-0203-layout-poppler.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
