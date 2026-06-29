# High-DPI Thumbnail Preview Fidelity 2026-06-26

Milestone: 0172

## Summary

Added a focused high-DPI thumbnail and preview gate for the Rust-native
renderer. The gate covers fine vector grid lines, small text, scaled image
content, transparency, scale-aware cache keys, and bounded raster allocation.

The renderer keeps the existing `max_edge` contract: it is an output ceiling,
not an implicit device-scale multiplier. Smaller pages are not upscaled beyond
their PDF page dimensions. The new high-DPI fixture uses a 480 x 360 page to
exercise higher-resolution preview output directly.

## Fixture Coverage

Added `fixtures/high-dpi-preview-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `high-dpi-preview` | 1 | New 480 x 360 high-DPI preview fixture. |
| `text-baseline` | 1 | Existing text page baseline at a high `max_edge` ceiling. |
| `vector-baseline` | 1 | Existing vector linework baseline at a high `max_edge` ceiling. |
| `image-baseline` | 1 | Existing image XObject baseline at a high `max_edge` ceiling. |
| `transparency-baseline` | 1 | Existing alpha compositing baseline at a high `max_edge` ceiling. |

New generated fixture:

- `fixtures/generated/high-dpi-preview-fidelity.pdf`

It is included in the main corpus manifest with `expected:native`.

## Native Coverage

Added:

- `native_page_cache_key_should_isolate_high_dpi_scale` verifies that caller-owned
  page cache keys include `max_edge`, so high-DPI renders cannot reuse stale
  lower-resolution output.
- `native_backend_should_render_generated_high_dpi_preview_fixtures` verifies
  expected output dimensions and visible-pixel coverage for the high-DPI fixture
  plus text, vector, image, and transparency baselines.
- `native_backend_should_enforce_high_dpi_raster_budget` verifies high-DPI page
  raster allocation remains bounded and maps budget failures to
  `renderer.memory-budget`.

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/high-dpi-preview-manifest.tsv \
  --include-family high-dpi-preview \
  --include-family text-baseline \
  --include-family vector-baseline \
  --include-family image-baseline \
  --include-family transparency-baseline \
  --fail-on-fallback \
  --max-edge 480 \
  --output target/highdpi-0172-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 5 | 5 | 0 | 0 |

## Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/high-dpi-preview-manifest.tsv \
  --include-family high-dpi-preview \
  --include-family text-baseline \
  --include-family vector-baseline \
  --include-family image-baseline \
  --include-family transparency-baseline \
  --max-edge 480 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/highdpi-0172-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `high-dpi-preview` | 1 | 1 | 0 | 0 | 0 | 250.611 | 250.611 | 691200 |
| `image-baseline` | 1 | 1 | 0 | 0 | 0 | 0.750 | 0.750 | 57600 |
| `text-baseline` | 1 | 1 | 0 | 0 | 0 | 1.259 | 1.259 | 192000 |
| `transparency-baseline` | 1 | 1 | 0 | 0 | 0 | 33.109 | 33.109 | 57600 |
| `vector-baseline` | 1 | 1 | 0 | 0 | 0 | 62.382 | 62.382 | 158400 |

## Batch Memory Profile

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/high-dpi-preview-manifest.tsv \
  --include-family high-dpi-preview \
  --include-family text-baseline \
  --include-family vector-baseline \
  --include-family image-baseline \
  --include-family transparency-baseline \
  --repetitions 2 \
  --max-edge 480 \
  --max-workers 2 \
  --max-in-flight-pixels 524288 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --output target/highdpi-0172-batch-memory.json
```

Result: 10 jobs, 10 native rendered, 0 fallbacks, 0 errors, 0 budget failures,
16.832 jobs/sec, p95 252.084 ms, max output 691200 bytes. RSS metrics are not
available on this host, so the recorded memory signal is the enforced
`max_in_flight_pixels = 524288` scheduler budget plus output-byte ceilings.

## Visual Comparison

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/high-dpi-preview-manifest.tsv \
  --include-family high-dpi-preview \
  --include-family text-baseline \
  --include-family vector-baseline \
  --include-family image-baseline \
  --include-family transparency-baseline \
  --max-edge 480 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/highdpi-0172-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 1 | 1 | 3 | 0 | 0 |

Exact match: `image-xobject`. Accepted drift: `vector-paths`. Blockers are
existing text/transparency/high-DPI fidelity deltas, not native support,
cache-key, or memory-budget failures.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo test -p ferrugo-native high_dpi -- --nocapture`
- `cargo test -p ferrugo-native native_page_cache_key_should_isolate_high_dpi_scale -- --nocapture`
- Native supported gate, benchmark, batch memory profile, and visual comparison
  commands listed above.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
