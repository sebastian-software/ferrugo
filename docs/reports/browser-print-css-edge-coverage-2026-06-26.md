# Browser Print CSS Edge Coverage 2026-06-26

Milestone: 0167

## Summary

Added a focused browser-print edge fixture slice and fixed native clipping scope
handling for saved graphics states. The new fixtures cover repeated/sticky
header geometry, clipped backgrounds, transformed cards, and mixed raster/vector
print output.

The new slice renders natively without fallback and compares exactly against
PDFium at the configured visual thresholds.

## Renderer Change

Clipping paths now carry the graphics-state depth and sibling scope id captured
when `W` or `W*` is interpreted. During rasterization, the active clip stack is
trimmed to the current paint item's scope before painting or adding a new clip.

This fixes a browser-print pattern where two independent `q ... W n ... Q`
regions at the same stack depth incorrectly intersected. The fix is bounded:
it stores two scalar fields per active clip and does not allocate clip masks.

Regression coverage:

- `rasterize_paths_should_restore_clip_with_graphics_state` verifies that two
  sibling clipping scopes paint independently.
- `native_backend_should_render_generated_browser_print_edge_fixtures` covers
  all four new browser-print edge fixtures through the native backend.

## Fixture Coverage

Added `fixtures/browser-print-edge-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `sticky-headers` | 1 | Repeated/sticky header bands and table grid geometry. |
| `clipped-backgrounds` | 1 | Independent clipped background panels in sibling graphics states. |
| `transformed-elements` | 1 | CSS-transform-like scaled card geometry. |
| `raster-vector` | 1 | Mixed image and vector chart content in print paint order. |
| `article` | 1 | Existing Chromium-style article print baseline. |
| `dashboard` | 1 | Existing Firefox-style dashboard print baseline. |
| `receipt-form` | 1 | Existing WebKit-style receipt/form print baseline. |

The four new generated PDFs are also included in the main corpus manifest under
`browser-print`.

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/browser-print-edge-manifest.tsv \
  --include-family sticky-headers \
  --include-family clipped-backgrounds \
  --include-family transformed-elements \
  --include-family raster-vector \
  --include-family article \
  --include-family dashboard \
  --include-family receipt-form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/browser-print-0167-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/browser-print-edge-manifest.tsv \
  --include-family sticky-headers \
  --include-family clipped-backgrounds \
  --include-family transformed-elements \
  --include-family raster-vector \
  --include-family article \
  --include-family dashboard \
  --include-family receipt-form \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/browser-print-0167-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `article` | 1 | 1 | 0 | 0 | 0 | 23.999 | 23.999 | 80000 |
| `clipped-backgrounds` | 1 | 1 | 0 | 0 | 0 | 111.900 | 111.900 | 102400 |
| `dashboard` | 1 | 1 | 0 | 0 | 0 | 41.555 | 41.555 | 72960 |
| `raster-vector` | 1 | 1 | 0 | 0 | 0 | 21.816 | 21.816 | 102400 |
| `receipt-form` | 1 | 1 | 0 | 0 | 0 | 52.565 | 52.565 | 68480 |
| `sticky-headers` | 1 | 1 | 0 | 0 | 0 | 57.632 | 57.632 | 102400 |
| `transformed-elements` | 1 | 1 | 0 | 0 | 0 | 48.826 | 48.826 | 102400 |

## Maintainer Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/browser-print-edge-manifest.tsv \
  --include-family sticky-headers \
  --include-family clipped-backgrounds \
  --include-family transformed-elements \
  --include-family raster-vector \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/browser-print-0167-new-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 4 | 0 | 0 | 0 | 0 |

All four new fixtures were exact at `max-edge 160`.

## Validation

- `cargo test -p pdfrust-render rasterize_paths_should_restore_clip_with_graphics_state -- --nocapture`
- `cargo test -p pdfrust-native browser_print_edge -- --nocapture`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
