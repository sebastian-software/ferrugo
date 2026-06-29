# Office Vector Effects And Clip Mask Fidelity 2026-06-26

Milestone: 0166

## Summary

Added a focused office-vector fixture slice and fixed a bounded native
clip/transparency gap: parent clipping paths now constrain transparency-group
compositing without allocating a separate clip mask.

The new slice renders natively without fallback. PDFium visual comparison still
classifies the subset as blocker-level pixel drift, so this milestone improves
native correctness and coverage but does not claim full visual parity.

## Renderer Change

`rasterize_transparency_group` now receives the active parent clip stack and
multiplies each composited group pixel by supersampled clip coverage. This keeps
memory bounded because it reuses the existing active clip paths and does not
allocate a page-sized mask.

Regression coverage:

- `native_backend_should_clip_transparency_group_to_parent_clip` checks that
  pixels outside the parent clip remain at the page background.
- `native_backend_should_render_generated_office_vector_effect_fixtures` covers
  the new grouped, nested-clip, clipped-transparency, and repeated-effect
  fixtures.

## Fixture Coverage

Added `fixtures/office-vector-effects-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `grouped-shapes` | 1 | Office-style grouped vector shapes and linework. |
| `nested-clips` | 2 | New nested clipping fixture plus existing spreadsheet clipped-cell baseline. |
| `clipped-transparency` | 1 | Parent clip applied to a transparency-group Form XObject. |
| `repeated-effects` | 2 | New repeated decorative vectors plus existing vector stress baseline. |

The four new generated PDFs are also included in the main corpus manifest under
`office-export`.

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/office-vector-effects-manifest.tsv \
  --include-family grouped-shapes \
  --include-family nested-clips \
  --include-family clipped-transparency \
  --include-family repeated-effects \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/office-vector-0166-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 6 | 6 | 0 | 0 |

## Clip And Memory Benchmark

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/office-vector-effects-manifest.tsv \
  --include-family nested-clips \
  --include-family clipped-transparency \
  --max-edge 200 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/office-vector-0166-clip-memory-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `clipped-transparency` | 1 | 1 | 0 | 0 | 0 | 117.697 | 117.697 | 110400 |
| `nested-clips` | 2 | 2 | 0 | 0 | 0 | 139.687 | 249.845 | 222400 |

## Maintainer Visual Comparison

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/office-vector-effects-manifest.tsv \
  --include-family grouped-shapes \
  --include-family nested-clips \
  --include-family clipped-transparency \
  --include-family repeated-effects \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/office-vector-0166-visual-diff.json
```

Result: 6 total, 0 native errors, 0 PDFium errors, 6 blockers.

Subsystem split:

| Subsystem | Blockers |
| --- | ---: |
| `vector-graphics` | 4 |
| `transparency` | 1 |
| `rendering-core` | 1 |

The blockers are visual-fidelity work, not runtime fallback failures. Current
typed unsupported boundaries remain outside this supported slice:
`graphics.pattern-shading` for mesh/pattern gaps and `graphics.transparency`
for luminosity soft masks or unsupported blend behavior.

## Validation

Passed:

- `cargo test -p ferrugo-native office_vector -- --nocapture`
- `cargo test -p ferrugo-native clip_transparency -- --nocapture`
- Native supported gate above.
- Clip/memory benchmark above.
- Maintainer PDFium visual comparison above.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
