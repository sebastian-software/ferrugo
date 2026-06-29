# Browser Print Corpus Refresh

Date: 2026-06-26.
Milestone: 0146.

## Summary

The browser-print family now includes 11 generated, deterministic fixtures. The
refresh adds current browser-print shaped reductions without depending on live
web pages or installed browsers.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `browser-chromium-article-print.pdf` | Chromium-style article print with CSS background blocks, image, table, text, and link annotation appearance. |
| `browser-firefox-dashboard-print.pdf` | Firefox-style dashboard print with CSS-grid-like cards, chart, table, and small labels. |
| `browser-webkit-receipt-form-print.pdf` | WebKit-style checkout receipt/form print with clipped overflow text, checkbox, totals, and barcode marker. |

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --fail-on-fallback --max-edge 160 --output target/browser-0146-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 11 | 11 | 0 | 0 |

## Repeat Native Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --repetitions 3 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/browser-0146-repeat-benchmark.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors | Budget failures | First mean ms | Repeat mean ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 11 | 11 | 0 | 0 | 0 | 26.252 | 26.066 |

## Visual Oracle

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/browser-0146-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 2 | 4 | 5 | 0 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `browser-chromium-article-print.pdf` | blocker | `rendering-core` | 2.553 | 19 | 0.105940 |
| `browser-firefox-dashboard-print.pdf` | blocker | `rendering-core` | 12.950 | 83 | 0.199419 |
| `browser-webkit-receipt-form-print.pdf` | blocker | `rendering-core` | 20.382 | 212 | 0.179271 |

The new rows render without PDFium fallback, but they intentionally increase
pressure on browser-style rendering-core fidelity: CSS background rectangles,
table/grid strokes, clipped overflow, chart geometry, and form-like controls.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `browser-chromium-article-print.pdf` | 1,981 |
| `browser-firefox-dashboard-print.pdf` | 1,806 |
| `browser-webkit-receipt-form-print.pdf` | 1,376 |
| **Total new PDF bytes** | **5,163** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- New fixture content is synthetic and has no private or customer document
  references.

## Backlog Impact

The refresh keeps browser-print in the core native-supported set while making
the next fidelity slices more representative:

1. Browser-style rendering-core parity for CSS backgrounds, table rules, and
   clipped overflow.
2. Text/font parity for web-font fallback and small labels.
3. Page geometry parity for UserUnit and print scaling drift.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo test -p ferrugo-cli corpus_manifest -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --fail-on-fallback --max-edge 160 --output target/browser-0146-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --repetitions 3 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/browser-0146-repeat-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/browser-0146-visual-diff.json
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/browser-chromium-article-print.pdf fixtures/generated/browser-firefox-dashboard-print.pdf fixtures/generated/browser-webkit-receipt-form-print.pdf
```
