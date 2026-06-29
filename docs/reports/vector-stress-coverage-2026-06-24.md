# Vector Stress Coverage 2026-06-24

This report records milestone 0072 coverage for vector-heavy chart and diagram
pages in the Rust-native thumbnail renderer.

## Implemented Slice

- Added `fixtures/generated/vector-stress.pdf`, a deterministic one-page chart
  stress fixture with nested rectangular clips, grid strokes, repeated filled
  bars, a cubic curve path, and small marker rectangles.
- Added render-crate coverage that the fixture produces a dense non-white
  raster at `160x120`.
- Added a segment-budget regression test that the fixture's cubic path fails
  predictably with `PathComplexityOverflow` when `max_flattened_segments` is set
  to `12`.
- Added native-backend coverage for the fixture.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p ferrugo-render vector_stress -- --nocapture
cargo test -p ferrugo-native vector_stress -- --nocapture
cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/vector-summary-0072.json
cargo run -p ferrugo-cli -- render-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/ferrugo-thumbnails/vector-stress-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli -- render-pdfium fixtures/generated/vector-stress.pdf --max-edge 160 --output target/ferrugo-thumbnails/vector-stress-pdfium.png
/usr/bin/time -p cargo run -p ferrugo-cli -- render-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/ferrugo-thumbnails/vector-stress-native-bench.png
```

All commands completed successfully. The portable timing run reported:

```text
real 3.11
user 2.83
sys 0.02
```

An attempted `/usr/bin/time -l` run produced useful timing output but exited
non-zero because this sandbox cannot read `sysctl kern.clockrate`; the portable
`time -p` run above is the recorded benchmark evidence.

The generated corpus summary reported 47 fixtures total, 45 native renders, 1
native fallback requirement for optional content policy, and 1 encrypted input
classification. The `report` family, including the new vector stress fixture,
rendered 10 of 10 fixtures natively.

Native and PDFium rendered `vector-stress.pdf` at `160x120`. Local RGBA
comparison reported mean absolute channel delta `0.417`, p95 channel delta `1`,
and max channel delta `69`. The larger deltas are edge anti-aliasing differences;
major chart layers are present in both outputs.

## Remaining Limits

- The rasterizer still scans the full page for each path item; the debug-build
  timing reflects that and should be optimized with tiled dirty regions or
  path-bounds clipping before larger vector-heavy documents become default-on.
- Clip restoration is still represented by display-list placeholders rather than
  a full graphics-state clip stack with `q`/`Q` pop semantics.
- CAD-grade precision, advanced stroke joins, and pathological path counts
  remain outside this milestone.
