# Scientific Report Fidelity 2026-06-25

Milestone: 0126.

## Decision

Scientific paper, equation/figure, long-report, and reference/footnote
thumbnails now have a focused native gate. The native renderer renders all
eight scientific-report manifest rows without PDFium fallback, errors, or
benchmark budget failures.

The native regression suite also samples a three-page long-report fixture
through the parallel scheduler, rendering pages 0 and 2 under an explicit
in-flight pixel budget.

PDFium remains a maintainer-only visual oracle. Current strict visual-diff
thresholds classify seven rows as fidelity blockers and one row as accepted
drift, with no native or PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `scientific-two-column-paper.pdf` | paper | two-column text blocks, figure, equation label, footnote |
| `scientific-equation-figure.pdf` | equation figure | equation-like text, vector figure panels, plotted marks |
| `reference-footnote-layout.pdf` | references footnotes | reference rows, footnote rule, dense small note text |
| `long-report-sampling.pdf` | long report | three pages, repeated tables, page sampling target |

`fixtures/scientific-report-manifest.tsv` combines these with existing embedded
font, Type3 symbol, multi-page report, and text-spacing baselines.

## Native Gate Evidence

Artifact: `target/scientific-0126-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `paper` | 2 | 2 | 0 | 0 |
| `equation-figure` | 2 | 2 | 0 | 0 |
| `long-report` | 2 | 2 | 0 | 0 |
| `references-footnotes` | 2 | 2 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

The native regression test also checks visible non-background pixel counts so
paper columns, figures, references, footnotes, and report tables cannot
silently collapse to empty output.

## Benchmark Evidence

Artifact: `target/scientific-0126-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `paper` | 2 | 2 | 14.140 | 26.732 | 0 |
| `equation-figure` | 2 | 2 | 20.565 | 39.730 | 0 |
| `long-report` | 2 | 2 | 33.792 | 40.615 | 0 |
| `references-footnotes` | 2 | 2 | 9.770 | 19.007 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/scientific-0126-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `paper` | 2 | 0 | 0 | 2 | 0 | 0 |
| `equation-figure` | 2 | 0 | 1 | 1 | 0 | 0 |
| `long-report` | 2 | 0 | 0 | 2 | 0 | 0 |
| `references-footnotes` | 2 | 0 | 0 | 2 | 0 | 0 |
| **Total** | **8** | **0** | **1** | **7** | **0** | **0** |

The remaining blockers are visual-fidelity work around text metrics, small
symbol placement, figure stroke antialiasing, and dense footnote/reference
layout, not native coverage failures.

## Follow-Up Backlog

- Improve small text metrics for dense two-column and reference layouts.
- Add producer-derived TeX and office whitepaper reductions once examples can
  be sanitized.
- Expand symbol/equation coverage beyond Type3 and Type1 proxy fixtures.
- Keep long-report sampling in future scheduler and memory-budget work.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/scientific-report-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p pdfrust-native scientific_report -- --nocapture
cargo test -p pdfrust-native long_report_pages -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/scientific-report-manifest.tsv --include-family paper --include-family equation-figure --include-family long-report --include-family references-footnotes --fail-on-fallback --max-edge 160 --output target/scientific-0126-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/scientific-report-manifest.tsv --include-family paper --include-family equation-figure --include-family long-report --include-family references-footnotes --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/scientific-0126-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/scientific-report-manifest.tsv --include-family paper --include-family equation-figure --include-family long-report --include-family references-footnotes --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/scientific-0126-visual-diff.json
```
