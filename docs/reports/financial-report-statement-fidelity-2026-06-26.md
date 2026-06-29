# Financial Report And Statement Fidelity

Date: 2026-06-26.
Milestone: 0149.

## Summary

The financial-document corpus now has a focused manifest at
`fixtures/financial-document-manifest.tsv`. It combines existing invoice and
statement baselines with new synthetic annual-report, cashflow, and KPI chart
summary reductions.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `financial-annual-report-page.pdf` | Annual-report snapshot with dense metrics, decimal columns, bar chart, and footer provenance text. |
| `financial-cashflow-statement.pdf` | Cashflow statement with strict decimal columns, negative values, table rules, and barcode marker. |
| `financial-chart-summary.pdf` | KPI chart summary with bar chart, line trend, legend text, and summary table. |

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --fail-on-fallback --max-edge 160 --output target/financial-0149-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 |

Supported family result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `annual-report` | 1 | 1 | 0 | 0 |
| `cashflow` | 1 | 1 | 0 | 0 |
| `chart-summary` | 1 | 1 | 0 | 0 |
| `invoice` | 2 | 2 | 0 | 0 |
| `invoice-table` | 1 | 1 | 0 | 0 |
| `report-statement` | 1 | 1 | 0 | 0 |
| `statement` | 1 | 1 | 0 | 0 |

## Dense Page Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/financial-0149-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `annual-report` | 1 | 34.964 | 34.964 | 0 |
| `cashflow` | 1 | 44.132 | 44.132 | 0 |
| `chart-summary` | 1 | 35.686 | 35.686 | 0 |
| `invoice` | 2 | 15.350 | 30.148 | 0 |
| `invoice-table` | 1 | 23.409 | 23.409 | 0 |
| `report-statement` | 1 | 26.392 | 26.392 | 0 |
| `statement` | 1 | 37.979 | 37.979 | 0 |

## Visual Oracle

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/financial-0149-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 0 | 0 | 8 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Blockers | Native errors |
| --- | ---: | ---: | ---: |
| `page-geometry` | 2 | 2 | 0 |
| `rendering-core` | 5 | 5 | 0 |
| `text-fonts` | 1 | 1 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `financial-annual-report-page.pdf` | blocker | `page-geometry` | 22.005 | 189 | 0.222094 |
| `financial-cashflow-statement.pdf` | blocker | `rendering-core` | 32.811 | 251 | 0.290727 |
| `financial-chart-summary.pdf` | blocker | `rendering-core` | 9.255 | 57 | 0.144307 |

These blockers are fidelity deltas, not native runtime fallbacks. They route to
dense table-rule parity, decimal text alignment, chart vector geometry, and
font/spacing follow-ups.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `financial-annual-report-page.pdf` | 1,760 |
| `financial-cashflow-statement.pdf` | 1,689 |
| `financial-chart-summary.pdf` | 1,533 |
| **Total new PDF bytes** | **4,982** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- `rg -n "private|customer|confidential|personal|production|PII|@" ...`
  returned only synthetic "no customer/no private data" fixture text plus an
  existing confidentiality clause in an unrelated contract fixture generator.
- New fixture content is synthetic and has no customer, accounting, or private
  financial-document source.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --fail-on-fallback --max-edge 160 --output target/financial-0149-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/financial-0149-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/financial-document-manifest.tsv --include-family invoice --include-family invoice-table --include-family statement --include-family report-statement --include-family annual-report --include-family cashflow --include-family chart-summary --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/financial-0149-visual-diff.json
cargo test -p ferrugo-native business_document -- --nocapture
cargo test -p ferrugo-native office_table -- --nocapture
cargo test -p ferrugo-render text_display_list -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/financial-annual-report-page.pdf fixtures/generated/financial-cashflow-statement.pdf fixtures/generated/financial-chart-summary.pdf
rg -n "private|customer|confidential|personal|production|PII|@" fixtures/corpus-manifest.tsv fixtures/financial-document-manifest.tsv scripts/generate_fixtures.py
```
