# Print Imposition Booklet Coverage 2026-06-29

Milestone: 0184.

## Summary

Added a focused print-imposition fixture slice for common booklet and n-up
print-preview documents. The new gate covers imposed sheet geometry, visible
crop marks, CropBox thumbnail selection, BleedBox/TrimBox context, fold marks,
and rotated slug text without requiring PDFium.

This is still a thumbnail-rendering boundary, not a print imposition engine or
prepress proofing suite.

## Fixture Coverage

Added `fixtures/print-imposition-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `booklet-spread` | 1 | Booklet cover spread with CropBox, BleedBox, TrimBox, crop marks, and center fold. |
| `n-up` | 1 | Four-up imposed sheet with page frames, registration cross, and rotated slug text. |
| `trim-bleed` | 1 | Existing trim/bleed page-box baseline. |
| `registration` | 1 | Existing registration mark and process-color baseline. |

The two new generated PDFs are also included in the main corpus manifest under
the `report` family with `print-imposition` feature tags.

## Page Box And Oracle Policy

Native thumbnails use CropBox as the visible page box when present and fall
back to MediaBox otherwise. BleedBox and TrimBox remain document context in
this milestone; callers still do not get explicit page-box selection modes.

The Poppler visual oracle now invokes `pdftoppm -cropbox` so the independent
reference uses the same visible-page boundary as native thumbnails. Without
that flag, Poppler rasterizes MediaBox content while the native renderer
correctly clips to CropBox, which makes page-box-sensitive fixtures look like
false visual blockers.

## Native Gate Evidence

Artifact: `target/print-imposition-0184-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `booklet-spread` | 1 | 1 | 0 | 0 |
| `n-up` | 1 | 1 | 0 | 0 |
| `registration` | 1 | 1 | 0 | 0 |
| `trim-bleed` | 1 | 1 | 0 | 0 |
| **Total** | **4** | **4** | **0** | **0** |

The native regression test renders the two new fixtures at their natural
geometry: `print-booklet-spread.pdf` resolves to `460 x 280` from CropBox and
`print-nup-imposed-sheet.pdf` resolves to `420 x 300` from MediaBox.

## Benchmark Evidence

Artifact: `target/print-imposition-0184-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `booklet-spread` | 1 | 1 | 12.679 | 12.679 | 0 |
| `n-up` | 1 | 1 | 12.965 | 12.965 | 0 |
| `registration` | 1 | 1 | 7.670 | 7.670 | 0 |
| `trim-bleed` | 1 | 1 | 24.910 | 24.910 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Poppler Visual Evidence

Artifact: `target/print-imposition-0184-poppler-visual-diff.json`

Thresholds: `--max-mae 5.5 --max-p95 16 --max-changed-ratio 0.13`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `booklet-spread` | 1 | 0 | 1 | 0 | 0 | 0 |
| `n-up` | 1 | 0 | 1 | 0 | 0 | 0 |
| **Total** | **2** | **0** | **2** | **0** | **0** | **0** |

The local threshold is narrower than the existing prepress boundary threshold
and is scoped to print-thumbnail antialiasing. Both fixtures had
`p95_channel_delta = 1`; the remaining drift comes from edge antialiasing and
small text rasterization, not missing geometry.

## Boundary Notes

- Supported: common booklet spreads and n-up imposed sheets as static
  thumbnails.
- Supported: visible crop marks, fold marks, registration crosses, and rotated
  slug text as ordinary vector/text content.
- Context only: BleedBox and TrimBox metadata.
- Out of scope: generating imposed layouts, trapping, separations,
  color-managed proofing, imposition marks semantics, and preflight validation.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/ferrugo-cli/src/main.rs crates/ferrugo-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/print-imposition-manifest.tsv scripts/generate_fixtures.py docs/corpus-taxonomy.md docs/backend/native.md docs/milestones/README.md docs/milestones/0184-print-shop-imposition-and-booklet-pdf-coverage.md docs/reports/print-imposition-booklet-coverage-2026-06-29.md
cargo test -p ferrugo-native print_imposition -- --nocapture
cargo test -p ferrugo-cli poppler --no-default-features
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/print-imposition-manifest.tsv --include-family booklet-spread --include-family n-up --include-family trim-bleed --include-family registration --fail-on-fallback --max-edge 160 --output target/print-imposition-0184-supported-gate.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/print-imposition-manifest.tsv --include-family booklet-spread --include-family n-up --include-family trim-bleed --include-family registration --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/print-imposition-0184-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/print-imposition-manifest.tsv --include-family booklet-spread --include-family n-up --max-edge 160 --max-mae 5.5 --max-p95 16 --max-changed-ratio 0.13 --timeout 30 --output target/print-imposition-0184-poppler-visual-diff.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
