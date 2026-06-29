# Layout Stress Corpus 2026-06-29

Milestone: 0189.

## Summary

Added a focused layout-stress corpus for dense report-style PDFs that combine
tables, two-column text, footnotes, small text, header/footer furniture, and
spreadsheet-like grids.

The native renderer can render every fixture in the focused set without PDFium
fallback. Visual parity is not complete: the Poppler oracle reports dense text
placement and footnote/table fidelity blockers, which is the intended signal
for this milestone rather than an accepted relaxation of thresholds.

## Fixture Coverage

Added `fixtures/generated/layout-columns-footnotes-table-stress.pdf`, a
single-page generated fixture with:

- two-column summary text blocks;
- a figure interrupt inside the right column;
- ruled table lines and numeric table cells;
- a footnote region below the table;
- header and footer page furniture.

Added `fixtures/layout-stress-manifest.tsv` with these families:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `layout-stress` | 1 | Intentional combined stress page for columns, table lines, footnotes, and furniture. |
| `dense-business-table` | 2 | Invoice and statement pages with dense table/ledger structure. |
| `spreadsheet-grid` | 1 | Dense repeated numeric grid and hairline rule coverage. |
| `two-column` | 1 | Scientific two-column text and figure placement. |
| `footnotes` | 1 | Small footnote-region text placement. |
| `page-furniture` | 1 | Header/footer/link furniture around report content. |

## Support Matrix

| Check | Result | Interpretation |
| --- | --- | --- |
| Native support gate | 7/7 native rendered, 0 fallbacks, 0 errors | Supported for server-side native thumbnail rendering. |
| Operator coverage | 1,332 operators scanned, 1,332 implemented, 0 partial, 0 unsupported | No content-stream operator gap in this focused set. |
| Benchmark budget | 7/7 native rendered, 0 errors, 0 budget failures | All families stayed below `max_ms = 1000` and `max_output_bytes = 1048576`. |
| Poppler visual oracle | 2 accepted drift, 4 blockers, 1 reference timeout | Visual fidelity gaps remain visible and should not be hidden by looser thresholds. |
| Semantic extraction | out of scope | This corpus does not validate table structure, reading order, or selection geometry. |

## Native Support Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --fail-on-fallback --max-edge 160 --output target/layout-stress-0189-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | --- |
| 7 | 7 | 0 | `{}` |

Family pass rates:

| Family | Total | Native rendered | Pass rate |
| --- | ---: | ---: | ---: |
| `dense-business-table` | 2 | 2 | 1.000 |
| `footnotes` | 1 | 1 | 1.000 |
| `layout-stress` | 1 | 1 | 1.000 |
| `page-furniture` | 1 | 1 | 1.000 |
| `spreadsheet-grid` | 1 | 1 | 1.000 |
| `two-column` | 1 | 1 | 1.000 |

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/layout-stress-0189-benchmark.json
```

Result:

| Family | Total | Native | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `dense-business-table` | 2 | 2 | 0 | 0 | 9.591 | 10.948 | 136960 |
| `footnotes` | 1 | 1 | 0 | 0 | 4.803 | 4.803 | 83200 |
| `layout-stress` | 1 | 1 | 0 | 0 | 21.214 | 21.214 | 78720 |
| `page-furniture` | 1 | 1 | 0 | 0 | 11.317 | 11.317 | 64000 |
| `spreadsheet-grid` | 1 | 1 | 0 | 0 | 8.296 | 8.296 | 70400 |
| `two-column` | 1 | 1 | 0 | 0 | 8.807 | 8.807 | 76800 |

## Operator Coverage

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --output target/layout-stress-0189-operator-coverage.json
```

Result: 7 fixtures scanned, 0 errors, 1,332 total operators, 1,332 implemented,
0 partial, 0 unsupported, 0 ignored. The new `layout-stress` family contributes
352 implemented operators.

## Visual Fidelity Review

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/layout-stress-0189-poppler-visual-diff.json
```

Thresholds reviewed:

| Metric | Threshold |
| --- | ---: |
| Mean absolute error | 6.000 |
| p95 channel delta | 24 |
| Changed ratio | 0.180 |

Result:

| Fixture | Status | Mean abs error | p95 delta | Changed ratio | Note |
| --- | --- | ---: | ---: | ---: | --- |
| `account-statement-ledger.pdf` | reference error | n/a | n/a | n/a | `pdftoppm` exceeded the 30s timeout. |
| `business-invoice-dense.pdf` | blocker | 5.605 | 31 | 0.099766 | p95 text/table drift exceeds threshold. |
| `layout-columns-footnotes-table-stress.pdf` | blocker | 16.190 | 107 | 0.232317 | Combined dense-layout stress exceeds all fidelity thresholds. |
| `office-report-header-footer-link.pdf` | accepted drift | 1.589 | 3 | 0.196500 | Low error magnitude; changed-ratio drift is broad but low intensity. |
| `reference-footnote-layout.pdf` | blocker | 8.586 | 21 | 0.064904 | Footnote-region mean error exceeds threshold. |
| `scientific-two-column-paper.pdf` | blocker | 12.533 | 105 | 0.153490 | Two-column text/figure placement drift remains visible. |
| `spreadsheet-dense-numeric-grid.pdf` | accepted drift | 0.417 | 0 | 0.047670 | Grid remains within practical visual drift. |

These results intentionally keep blockers visible. The native renderer is
usable for generating thumbnails for the set, but dense-layout visual parity is
not complete and should feed targeted renderer work instead of threshold
relaxation.

## Renderer Gap List

1. `text-fonts`: dense small text and two-column glyph placement still diverge
   materially from Poppler/PDFium-class rendering.
2. `rendering-core`: ruled table/grid strokes are supported, but dense combined
   pages make small coordinate and antialiasing differences more visible.
3. `footnotes`: small text below table rules needs explicit regression
   visibility because it can be easy to hide under coarse page-level metrics.
4. `reference-oracle`: one dense statement fixture timed out in Poppler at 30s;
   future review should either raise oracle timeout for that family or split it
   into a separate slow-reference lane.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs scripts/generate_fixtures.py fixtures/corpus-manifest.tsv fixtures/layout-stress-manifest.tsv docs/corpus-taxonomy.md docs/milestones/0189-layout-stress-corpus-for-tables-columns-and-footnotes.md docs/milestones/README.md docs/reports/layout-stress-corpus-2026-06-29.md
cargo test -p ferrugo-native scientific_report -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --fail-on-fallback --max-edge 160 --output target/layout-stress-0189-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/layout-stress-0189-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --output target/layout-stress-0189-operator-coverage.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/layout-stress-manifest.tsv --include-family layout-stress --include-family dense-business-table --include-family spreadsheet-grid --include-family two-column --include-family footnotes --include-family page-furniture --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/layout-stress-0189-poppler-visual-diff.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
