# Memory Arena And Scratch Buffer Audit 2026-06-26

Milestone: 0158.

## Decision

Do not add a custom allocator or unsafe arena. The current renderer already
uses explicit byte, pixel, cache, and display-list budgets in the hot paths.
The useful 0158 change is narrower: bound text raster scratch retention after
an unusually large fallback text run so later small text runs do not carry a
large vector capacity for the rest of the rasterization pass.

## Allocation Hotspots

| Area | Current state | 0158 action |
| --- | --- | --- |
| Parser and object loading | Borrowed input plus bounded decoded stream buffers. | No arena; keep byte budgets and typed errors. |
| Display-list building | Bounded item, path, text-run, CMap, font, image, ICC, and mesh limits. | No broad change. |
| Text fallback rasterization | Pass-local `TextRasterScratch` and `GlyphBitmapCache`. | Added bounded capacity reset for oversized scratch. |
| Image conversion | Decoded source samples retained once; per-draw `ImageSampleCache` avoids repeated conversion for repeated target samples. | No change. |
| Path rasterization | Device bounds limit per-item pixel visits; pattern cache is entry bounded. | No change. |
| Transparency groups | Intermediate raster pixels are explicitly bounded. | No change. |

## Implemented Change

`TextRasterScratch` now has a retained-atom ceiling of `4096`. Normal text runs
still reuse scratch capacity. If a large run grows the scratch vector above the
ceiling, the next run whose expected glyph count fits under the ceiling replaces
the vector with a smaller allocation before filling it.

This keeps reset semantics explicit:

- Scratch lifetime remains one rasterization pass.
- The retained capacity has a deterministic ceiling.
- Large consecutive text runs can still reuse their allocation.
- Small text runs after a large one release oversized capacity.
- Rendering output is unchanged.

## Measurement Evidence

The focused unit test `text_raster_scratch_should_release_oversized_capacity_before_small_run`
drives the before/after shape directly: a large fallback text run grows scratch
capacity, then a small run forces the new bounded reset and still emits the
same glyph atom.

Low-memory benchmark artifact: `target/scratch-0158-low-memory-benchmark.json`.

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `common` | 2 | 2 | 15.712 | 30.781 | 0 |
| `repeated-resources` | 1 | 1 | 16.576 | 16.576 | 0 |
| `scan` | 1 | 1 | 42.692 | 42.692 | 0 |
| `vector-stress` | 1 | 1 | 193.960 | 193.960 | 0 |

Native supported gate artifact: `target/scratch-0158-supported-gate.json`.
It rendered 5/5 low-memory-profile fixtures with 0 fallbacks and 0 errors.

## Visual Subset

Visual oracle artifact: `target/scratch-0158-visual-diff.json`.

The subset ran `common` and `repeated-resources` fixtures through the PDFium
oracle. It produced no native errors and no PDFium errors. The 3/3 blocker
rows remain in the known `text-fonts` and `rendering-core` parity areas; this
scratch-retention slice does not attempt to change visual fidelity.

## Validation Commands

```text
cargo fmt --check
cargo test -p ferrugo-render text_raster_scratch -- --nocapture
cargo test -p ferrugo-render rasterize_text -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family scan --include-family vector-stress --include-family repeated-resources --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/scratch-0158-low-memory-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family scan --include-family vector-stress --include-family repeated-resources --fail-on-fallback --max-edge 160 --native-profile low-memory --output target/scratch-0158-supported-gate.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family repeated-resources --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/scratch-0158-visual-diff.json
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.
