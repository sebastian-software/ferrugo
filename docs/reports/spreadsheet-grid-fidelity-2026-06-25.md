# Spreadsheet Grid Fidelity 2026-06-25

Milestone: 0123.

## Decision

Spreadsheet and dense-table thumbnails now have a focused native gate. The
native renderer renders all seven spreadsheet-grid manifest rows without PDFium
fallback, errors, or benchmark budget failures.

PDFium remains a maintainer-only visual oracle. Current strict visual-diff
thresholds classify all seven rows as fidelity blockers, with no native or
PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `spreadsheet-frozen-header.pdf` | frozen header | frozen row/column guide lines, thin grid strokes, small cells |
| `spreadsheet-dense-numeric-grid.pdf` | dense grid | dense numeric cells, thin repeated grid lines, small text |
| `spreadsheet-clipped-cells.pdf` | clipped cells | per-cell clipping with overflowing text labels |
| `spreadsheet-vector-stress-grid.pdf` | stress grid | high-line-count spreadsheet grid for repeated stroke workload |

`fixtures/spreadsheet-grid-manifest.tsv` combines these with existing office
table, ledger, and vector-stress baselines.

## Native Gate Evidence

Artifact: `target/spreadsheet-0123-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `frozen-header` | 2 | 2 | 0 | 0 |
| `dense-grid` | 2 | 2 | 0 | 0 |
| `clipped-cells` | 1 | 1 | 0 | 0 |
| `stress-grid` | 2 | 2 | 0 | 0 |
| **Total** | **7** | **7** | **0** | **0** |

The native regression test also checks dense visible pixel counts so grid/text
content cannot silently collapse to a near-empty thumbnail.

## Benchmark Evidence

Artifact: `target/spreadsheet-0123-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `frozen-header` | 2 | 2 | 22.697 | 24.268 | 0 |
| `dense-grid` | 2 | 2 | 31.252 | 38.156 | 0 |
| `clipped-cells` | 1 | 1 | 21.223 | 21.223 | 0 |
| `stress-grid` | 2 | 2 | 110.618 | 191.991 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/spreadsheet-0123-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `frozen-header` | 2 | 0 | 0 | 2 | 0 | 0 |
| `dense-grid` | 2 | 0 | 0 | 2 | 0 | 0 |
| `clipped-cells` | 1 | 0 | 0 | 1 | 0 | 0 |
| `stress-grid` | 2 | 0 | 0 | 2 | 0 | 0 |
| **Total** | **7** | **0** | **0** | **7** | **0** | **0** |

The remaining blockers are visual-fidelity work around text metrics, thin-line
antialiasing, and clipping edge placement, not native coverage failures.

## Follow-Up Backlog

- Tune thin-stroke antialiasing against PDFium for table borders.
- Improve small-text placement for dense spreadsheet exports.
- Add producer-derived spreadsheet reductions once private examples can be
  sanitized.
- Reuse the stress-grid fixture in future raster hot-path profiling.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/spreadsheet-grid-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p ferrugo-native spreadsheet_grid -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --fail-on-fallback --max-edge 160 --output target/spreadsheet-0123-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/spreadsheet-0123-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/spreadsheet-grid-manifest.tsv --include-family frozen-header --include-family dense-grid --include-family clipped-cells --include-family stress-grid --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/spreadsheet-0123-visual-diff.json
```
