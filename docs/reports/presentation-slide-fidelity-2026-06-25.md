# Presentation Slide Fidelity 2026-06-25

Milestone: 0122.

## Decision

Common slide-export PDFs now have a dedicated native gate. The native renderer
renders all seven presentation-slide manifest rows without PDFium fallback,
errors, or benchmark budget failures.

PDFium remains useful as a maintainer-only visual oracle. Current strict
visual-diff thresholds classify two rows as exact and five as blockers, with no
native or PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `slide-title-gradient.pdf` | title slide | axial gradient background, translucent shadow block, positioned text |
| `slide-layered-image-shadow.pdf` | image slide | scaled Image XObject, tint overlay, soft shadow block, text |
| `slide-rotated-callout.pdf` | chart slide | chart bars, translucent panel, rotated label text |
| `slide-speaker-notes-page.pdf` | notes page | slide thumbnail, notes rule lines, positioned notes text |

`fixtures/presentation-slide-manifest.tsv` combines these with existing
presentation baselines for Form XObjects, default-visible optional content, and
spot-color vectors.

## Native Gate Evidence

Artifact: `target/presentation-0122-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `title-slide` | 3 | 3 | 0 | 0 |
| `image-slide` | 1 | 1 | 0 | 0 |
| `chart-slide` | 2 | 2 | 0 | 0 |
| `notes-page` | 1 | 1 | 0 | 0 |
| **Total** | **7** | **7** | **0** | **0** |

The known `optional-content-ocmd.pdf` fallback remains outside this common
slide gate and continues to represent the richer optional-content policy
boundary.

## Benchmark Evidence

Artifact: `target/presentation-0122-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `title-slide` | 3 | 3 | 7.976 | 10.189 | 0 |
| `image-slide` | 1 | 1 | 17.937 | 17.937 | 0 |
| `chart-slide` | 2 | 2 | 14.455 | 21.150 | 0 |
| `notes-page` | 1 | 1 | 25.515 | 25.515 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/presentation-0122-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `title-slide` | 3 | 2 | 0 | 1 | 0 | 0 |
| `image-slide` | 1 | 0 | 0 | 1 | 0 | 0 |
| `chart-slide` | 2 | 0 | 0 | 2 | 0 | 0 |
| `notes-page` | 1 | 0 | 0 | 1 | 0 | 0 |
| **Total** | **7** | **2** | **0** | **5** | **0** | **0** |

The remaining blockers are visual-fidelity work, not native coverage failures.
They mostly exercise slide-specific text placement, shadow/tint compositing, and
chart-callout geometry.

## Follow-Up Backlog

- Tune text positioning and glyph metrics on slide title and notes text.
- Add more realistic image-heavy slides once private examples can be reduced to
  shareable fixtures.
- Add a larger full-bleed image fixture when image downsampling and memory
  optimization work reaches milestone 0137.
- Keep OCMD policy fixtures in the broader presentation family, but outside
  this common slide-export native gate.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/presentation-slide-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p ferrugo-native presentation_slide -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/presentation-slide-manifest.tsv --include-family title-slide --include-family image-slide --include-family chart-slide --include-family notes-page --fail-on-fallback --max-edge 160 --output target/presentation-0122-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/presentation-slide-manifest.tsv --include-family title-slide --include-family image-slide --include-family chart-slide --include-family notes-page --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/presentation-0122-benchmark.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/presentation-slide-manifest.tsv --include-family title-slide --include-family image-slide --include-family chart-slide --include-family notes-page --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/presentation-0122-visual-diff.json
```
