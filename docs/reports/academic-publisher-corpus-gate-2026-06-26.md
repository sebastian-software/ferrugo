# Academic Publisher Corpus Gate

Date: 2026-06-26.
Milestone: 0150.

## Summary

The academic publisher corpus now has a focused manifest at
`fixtures/academic-publisher-manifest.tsv`. It combines the existing
scientific-report reductions with new publisher article, equation/symbol, and
references/appendix pages.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `academic-publisher-first-page.pdf` | Publisher-style article first page with DOI text, two columns, figure panel, and small provenance text. |
| `academic-equation-symbols-page.pdf` | Equation and symbol placement page with ASCII math proxies and vector figure marks. |
| `academic-references-appendix.pdf` | References, appendix, and footnote layout with dense small text blocks. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --fail-on-fallback --max-edge 160 --output target/academic-0150-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 9 | 9 | 0 | 0 |

Supported family result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `equation-figure` | 3 | 3 | 0 | 0 |
| `long-report` | 1 | 1 | 0 | 0 |
| `paper` | 1 | 1 | 0 | 0 |
| `publisher-article` | 1 | 1 | 0 | 0 |
| `references-footnotes` | 3 | 3 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/academic-0150-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `equation-figure` | 3 | 30.290 | 48.881 | 0 |
| `long-report` | 1 | 39.901 | 39.901 | 0 |
| `paper` | 1 | 26.845 | 26.845 | 0 |
| `publisher-article` | 1 | 26.702 | 26.702 | 0 |
| `references-footnotes` | 3 | 14.745 | 25.035 | 0 |

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/academic-0150-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 9 | 0 | 1 | 8 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `page-geometry` | 2 | 0 | 2 | 0 |
| `rendering-core` | 5 | 0 | 5 | 0 |
| `text-fonts` | 2 | 1 | 1 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `academic-equation-symbols-page.pdf` | blocker | `page-geometry` | 9.341 | 44 | 0.119211 |
| `academic-publisher-first-page.pdf` | blocker | `page-geometry` | 18.953 | 105 | 0.176875 |
| `academic-references-appendix.pdf` | blocker | `rendering-core` | 20.332 | 159 | 0.153413 |

These blockers are fidelity deltas, not native runtime fallbacks. They route to
small-text metrics, multi-column geometry, equation/symbol placement, and
figure/vector antialiasing follow-ups.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `academic-publisher-first-page.pdf` | 2,584 |
| `academic-equation-symbols-page.pdf` | 1,135 |
| `academic-references-appendix.pdf` | 1,595 |
| **Total new PDF bytes** | **5,314** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- `rg -n "private|customer|confidential|personal|production|PII|@" ...`
  returned only synthetic "no customer/no private data" fixture text plus an
  existing confidentiality clause in an unrelated contract fixture generator.
- New fixture content is synthetic and has no real manuscript, publisher, or
  private research source.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --fail-on-fallback --max-edge 160 --output target/academic-0150-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/academic-0150-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/academic-publisher-manifest.tsv --include-family paper --include-family publisher-article --include-family equation-figure --include-family references-footnotes --include-family long-report --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/academic-0150-visual-diff.json
cargo test -p pdfrust-native scientific_report -- --nocapture
cargo test -p pdfrust-native font_subset -- --nocapture
cargo test -p pdfrust-render text_display_list -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/academic-publisher-first-page.pdf fixtures/generated/academic-equation-symbols-page.pdf fixtures/generated/academic-references-appendix.pdf
rg -n "private|customer|confidential|personal|production|PII|@" fixtures/corpus-manifest.tsv fixtures/academic-publisher-manifest.tsv scripts/generate_fixtures.py
```
