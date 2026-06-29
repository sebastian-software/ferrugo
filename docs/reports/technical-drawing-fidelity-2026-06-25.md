# Technical Drawing Fidelity 2026-06-25

Milestone: 0124.

## Decision

Technical drawing and CAD-style thumbnails now have a focused native gate. The
native renderer renders all eight technical drawing manifest rows without
PDFium fallback, errors, or benchmark budget failures.

PDFium remains a maintainer-only visual oracle. Current strict visual-diff
thresholds classify six rows as fidelity blockers and two rows as exact
matches, with no native or PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `technical-linework-dimensions.pdf` | linework | fine outlines, dashed construction grid, dimension lines, small labels |
| `technical-hatch-clipping.pdf` | hatch clipping | clipped hatch lines, dashed section centerline, section label |
| `technical-large-coordinate-plan.pdf` | large coordinate | large page box, downscaled linework, grid, dimension label |
| `technical-repeated-symbols.pdf` | repeated symbols | repeated small path symbols, line joins, thin strokes |

`fixtures/technical-drawing-manifest.tsv` combines these with existing dashed
stroke, clipped path, vector-stress, and UserUnit geometry baselines.

## Native Gate Evidence

Artifact: `target/technical-0124-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `linework` | 2 | 2 | 0 | 0 |
| `hatch-clipping` | 2 | 2 | 0 | 0 |
| `large-coordinate` | 2 | 2 | 0 | 0 |
| `repeated-symbols` | 2 | 2 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

The native regression test also checks visible non-background pixel counts so
fine technical linework cannot silently collapse to a near-empty thumbnail.

## Benchmark Evidence

Artifact: `target/technical-0124-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `hatch-clipping` | 2 | 2 | 175.969 | 304.313 | 0 |
| `large-coordinate` | 2 | 2 | 37.774 | 70.266 | 0 |
| `linework` | 2 | 2 | 32.309 | 63.416 | 0 |
| `repeated-symbols` | 2 | 2 | 126.073 | 190.556 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/technical-0124-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `linework` | 2 | 1 | 0 | 1 | 0 | 0 |
| `hatch-clipping` | 2 | 1 | 0 | 1 | 0 | 0 |
| `large-coordinate` | 2 | 0 | 0 | 2 | 0 | 0 |
| `repeated-symbols` | 2 | 0 | 0 | 2 | 0 | 0 |
| **Total** | **8** | **2** | **0** | **6** | **0** | **0** |

The remaining blockers are visual-fidelity work around thin-stroke
antialiasing, hatch clipping edge placement, large-coordinate/UserUnit
geometry, and repeated-symbol path output, not native coverage failures.

## Follow-Up Backlog

- Tune thin-stroke antialiasing against PDFium for engineering linework.
- Improve hatch clipping fidelity at polygon edges.
- Tighten large-coordinate and UserUnit transform parity.
- Profile repeated-symbol path workloads for future raster hot-path work.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/technical-drawing-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p ferrugo-native technical_drawing -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --fail-on-fallback --max-edge 160 --output target/technical-0124-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/technical-0124-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/technical-0124-visual-diff.json
```
