# Longform Text Fidelity 2026-06-25

Milestone: 0127.

## Decision

Book, manual, ebook, and repeated-resource longform thumbnails now have a
focused native gate. The native renderer renders all eight longform manifest
rows without PDFium fallback, errors, or benchmark budget failures.

The native regression suite also verifies book metadata for page labels and
outlines, samples frontmatter/chapter/appendix pages through the parallel
scheduler, and checks the renderer exposes bounded cache diagnostics for
longform workloads.

PDFium remains a maintainer-only visual oracle. Current strict visual-diff
thresholds classify seven rows as fidelity blockers and one row as an exact
match, with no native or PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `book-frontmatter-page-labels.pdf` | book | five pages, front matter, chapter labels, outlines |
| `manual-illustrated-chapter.pdf` | manual | procedure text, illustration panel, table layout |
| `ebook-narrow-longform.pdf` | ebook | narrow page, longform text blocks, footer marker |
| `longform-repeated-resources.pdf` | repeated resources | three pages, shared Type1 font, shared Image XObject |

`fixtures/longform-text-manifest.tsv` combines these with existing metadata,
mixed text/image, simple text, and long-report sampling baselines.

## Native Gate Evidence

Artifact: `target/longform-0127-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `book` | 2 | 2 | 0 | 0 |
| `manual` | 2 | 2 | 0 | 0 |
| `ebook` | 2 | 2 | 0 | 0 |
| `repeated-resources` | 2 | 2 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

The native regression test also checks visible non-background pixel counts so
longform text structure, illustrations, and repeated resource pages cannot
silently collapse to empty output.

## Metadata And Sampling Evidence

The book fixture resolves five pages, three outline items, and page labels:
`i`, `ii`, `Ch-1`, `Ch-2`, and `Ch-3`.

The parallel sampling test renders book pages 0, 2, and 4, plus repeated
resource pages 0 and 2, under an explicit `320 * 320 * 2` in-flight pixel
budget. Cache diagnostics confirm bounded font fallback, image, total image,
and display-item budgets are exposed.

## Benchmark Evidence

Artifact: `target/longform-0127-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `book` | 2 | 2 | 16.238 | 21.421 | 0 |
| `manual` | 2 | 2 | 32.734 | 32.959 | 0 |
| `ebook` | 2 | 2 | 7.489 | 14.386 | 0 |
| `repeated-resources` | 2 | 2 | 28.230 | 40.232 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/longform-0127-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `book` | 2 | 1 | 0 | 1 | 0 | 0 |
| `manual` | 2 | 0 | 0 | 2 | 0 | 0 |
| `ebook` | 2 | 0 | 0 | 2 | 0 | 0 |
| `repeated-resources` | 2 | 0 | 0 | 2 | 0 | 0 |
| **Total** | **8** | **1** | **0** | **7** | **0** | **0** |

The remaining blockers are visual-fidelity work around small text metrics,
manual illustration stroke placement, ebook text density, and image/text
antialiasing differences, not native coverage failures.

## Follow-Up Text Metrics 2026-06-28

The native renderer now uses deterministic Times-Roman advance widths for
standard-base serif fallback text instead of the generic 500-unit glyph advance.
The focused Poppler check was rerun for the `book` family:

```sh
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/longform-text-manifest.tsv \
  --include-family book \
  --max-edge 160 \
  --timeout 20 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/book-times-poppler-visual-diff.json
```

Result: 2 total, 1 exact, 0 accepted drift, 1 blocker, 0 native errors,
0 reference errors, 0 both errors. The `book-frontmatter-page-labels.pdf`
blocker remains, but the Times-Roman width table reduced changed ratio from
0.061207 to 0.060614 and MAE from 5.960 to 5.911. The remaining blocker is
still glyph-shape and text-antialiasing fidelity, not standard serif advance
spacing.

## Follow-Up Backlog

- Expand sanitized producer-derived books, manuals, and ebook exports.
- Add page-cache reuse counters once runtime cache telemetry exists.
- Improve longform small-text metrics against PDFium.
- Reuse repeated-resource pages in future high-page-count and cache milestones.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/longform-text-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p pdfrust-native longform -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/longform-text-manifest.tsv --include-family book --include-family manual --include-family ebook --include-family repeated-resources --fail-on-fallback --max-edge 160 --output target/longform-0127-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/longform-text-manifest.tsv --include-family book --include-family manual --include-family ebook --include-family repeated-resources --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/longform-0127-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/longform-text-manifest.tsv --include-family book --include-family manual --include-family ebook --include-family repeated-resources --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/longform-0127-visual-diff.json
```
