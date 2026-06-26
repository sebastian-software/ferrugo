# Office Suite Regression Corpus Refresh

Date: 2026-06-26.
Milestone: 0145.

## Summary

The office-export family now includes 47 generated, license-safe fixtures. This
refresh adds mixed Office-suite documents that are closer to Word, Excel,
PowerPoint, and LibreOffice exports while keeping every new fixture tiny and
reproducible from `scripts/generate_fixtures.py`.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `office-report-header-footer-link.pdf` | Word/LibreOffice-style report with header, footer, page background, logo image, table, and link annotation appearance. |
| `office-spreadsheet-chart-comments.pdf` | Spreadsheet page with dense cells, thin grid strokes, chart area, and visible comment markers. |
| `office-presentation-handout.pdf` | Presentation handout with slide thumbnails, embedded image, chart bars, and speaker notes. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family office-export --fail-on-fallback --max-edge 160 --output target/office-0145-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `office-export` | 47 | 47 | 0 | 0 |

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family office-export --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/office-0145-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 47 | 0 | 3 | 44 | 0 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `office-report-header-footer-link.pdf` | blocker | `rendering-core` | 11.137 | 125 | 0.236333 |
| `office-spreadsheet-chart-comments.pdf` | blocker | `rendering-core` | 17.989 | 145 | 0.300676 |
| `office-presentation-handout.pdf` | blocker | `rendering-core` | 5.957 | 13 | 0.133917 |

The blocker status is a fidelity signal, not a runtime fallback: all three new
fixtures render natively without PDFium. The deltas reinforce the existing
office backlog around text metrics, table/grid strokes, chart/layout details,
and image/vector composition.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `office-report-header-footer-link.pdf` | 2,209 |
| `office-spreadsheet-chart-comments.pdf` | 2,383 |
| `office-presentation-handout.pdf` | 1,428 |
| **Total new PDF bytes** | **6,020** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- The changed fixture metadata and generator are synthetic and contain no
  private or customer document references.

## Backlog Impact

This refresh keeps the native-only office gate green while increasing the
visual oracle pressure on `rendering-core`. The next office fidelity slices
should focus on:

1. Dense spreadsheet/table line placement and clipping.
2. Office report text metrics and header/footer composition.
3. Chart and handout layout parity against PDFium.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo test -p pdfrust-cli corpus_manifest -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family office-export --fail-on-fallback --max-edge 160 --output target/office-0145-supported-gate.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family office-export --max-edge 120 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/office-0145-visual-diff.json
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/office-report-header-footer-link.pdf fixtures/generated/office-spreadsheet-chart-comments.pdf fixtures/generated/office-presentation-handout.pdf
```
