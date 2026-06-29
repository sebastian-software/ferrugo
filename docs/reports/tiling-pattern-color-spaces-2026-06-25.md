# Tiling Patterns And Pattern Color Spaces

Date: 2026-06-25
Milestone: 0107

## Summary

Milestone 0107 extends the Rust-native renderer from colored tiling patterns to
common uncolored tiling patterns selected through PDF `[/Pattern <base>]` color
spaces. The display-list interpreter now accepts pattern color-space resources,
applies caller-supplied DeviceGray, DeviceRGB, or DeviceCMYK paint operands, and
passes the resolved color into the pattern cell renderer.

Pattern cell samples are cached per rasterization pass with a small
transform-aware entry budget. Setting the budget to zero disables retained cache
entries while still allowing the current render operation to proceed.

## Implementation

- Added `PatternBaseColorSpace` for DeviceGray, DeviceRGB, and DeviceCMYK
  uncolored pattern operands.
- Added `TilingPatternPaint` and decoded `/PaintType 1` as colored and
  `/PaintType 2` as uncolored.
- Added Pattern color-space resource decoding for `[/Pattern /DeviceGray]`,
  `[/Pattern /DeviceRGB]`, and `[/Pattern /DeviceCMYK]`, including common
  aliases.
- Added a rasterization-pass `PatternCellCache` keyed by resource name, paint
  mode, caller color for uncolored patterns, and quantized transform scale.
- Added `PathRasterOptions::max_pattern_cell_cache_entries` with a default of
  32 cached cells.
- Added generated fixture
  `fixtures/generated/uncolored-tiling-pattern.pdf`.

## Evidence

Supported-family native-only gate:

- Total: 41
- Native rendered: 41
- Fallback required: 0
- Browser-print: 6/6 native rendered
- Form: 12/12 native rendered
- Office-export: 23/23 native rendered
- Artifact: `target/pattern-0107-supported-gate.json`

Native benchmark:

- Total fixtures: 92
- Native rendered: 86
- Fallback required: 5
- Errors: 1
- Budget failures: 6
- Artifact: `target/pattern-0107-benchmark.json`

Pattern fixture benchmark results:

| Fixture | Native status | Mean ms | Output bytes | Budget violations |
| --- | --- | ---: | ---: | --- |
| `tiling-pattern.pdf` | native_rendered | 31.006 | 57600 | none |
| `uncolored-tiling-pattern.pdf` | native_rendered | 24.965 | 57600 | none |

PDFium visual comparison:

- Artifact: `target/pattern-0107-visual-diff.json`
- `tiling-pattern.pdf`: exact match, MAE 0.000, changed ratio 0.000000, p95
  channel delta 0.
- `uncolored-tiling-pattern.pdf`: exact match, MAE 0.000, changed ratio
  0.000000, p95 channel delta 0.

The full visual-diff corpus still contains unrelated blockers in other
subsystems. The two pattern fixtures added or exercised by this milestone are
exact PDFium matches.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-render`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/pattern-0107-benchmark.json`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/pattern-0107-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium/out/ferrugo-dylib:/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/pattern-0107-visual-diff.json`

## Follow-Ups

- Expand uncolored pattern fixtures beyond DeviceRGB to cover DeviceGray and
  DeviceCMYK pattern color spaces.
- Revisit pattern rendering when shading patterns and mesh shadings are handled
  by later milestones.
- Add longer-lived page or document cache integration only after multi-page
  renderer sessions establish their cache ownership boundaries.
