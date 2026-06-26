# Geospatial Map PDF Rendering Coverage

Date: 2026-06-26.
Milestone: 0152.

## Summary

The map rendering corpus now has a focused manifest at
`fixtures/map-rendering-manifest.tsv`. It separates supported visual map
features from the intentionally unsupported OCMD optional-content membership
policy boundary.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `map-raster-tile-routes.pdf` | Repeated raster tile Image XObjects, grid lines, vector route, markers, and labels. |
| `map-transparent-zoning-overlay.pdf` | Transparent zoning fills over grid and route vectors. |
| `map-optional-layer-policy.pdf` | Simple OCG zoning layer disabled by default with deterministic base-map rendering. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --fail-on-fallback --max-edge 160 --output target/map-0152-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 7 | 7 | 0 | 0 |

## Optional Content Boundary

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family unsupported-optional-policy --max-edge 160 --output target/map-0152-unsupported-optional-policy.json
```

Result:

| Family | Total | Native rendered | Fallback required | Fallback category |
| --- | ---: | ---: | ---: | --- |
| `unsupported-optional-policy` | 1 | 0 | 1 | `graphics.optional-content` |

Simple OCG layer on/off behavior is supported and deterministic. OCMD
membership policy remains an explicit unsupported boundary.

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/map-0152-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `map` | 1 | 103.463 | 103.463 | 0 |
| `optional-layer` | 1 | 44.507 | 44.507 | 0 |
| `pattern-layer` | 1 | 32.110 | 32.110 | 0 |
| `raster-tile` | 1 | 255.410 | 255.410 | 0 |
| `vector-layer` | 1 | 47.516 | 47.516 | 0 |
| `zoning-overlay` | 2 | 46.305 | 63.288 | 0 |

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/map-0152-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 2 | 0 | 5 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Exact | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `rendering-core` | 5 | 0 | 5 | 0 |
| `vector-graphics` | 2 | 2 | 0 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `map-optional-layer-policy.pdf` | blocker | `rendering-core` | 4.606 | 20 | 0.116121 |
| `map-raster-tile-routes.pdf` | blocker | `rendering-core` | 5.589 | 31 | 0.126480 |
| `map-transparent-zoning-overlay.pdf` | blocker | `rendering-core` | 5.463 | 30 | 0.183716 |

These blockers are visual-fidelity deltas, not native runtime fallbacks. The
exact `clipped-paths.pdf` and `tiling-pattern.pdf` rows remain useful map-layer
control baselines.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `map-raster-tile-routes.pdf` | 1,413 |
| `map-transparent-zoning-overlay.pdf` | 1,170 |
| `map-optional-layer-policy.pdf` | 1,124 |
| **Total new PDF bytes** | **3,707** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- `rg -n "private|customer|confidential|personal|production|PII|@" ...`
  returned only synthetic "no customer/no private data" fixture text plus an
  existing confidentiality clause in an unrelated contract fixture generator.
- New fixture content is synthetic and has no real map, facility, customer, or
  geospatial data source.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --fail-on-fallback --max-edge 160 --output target/map-0152-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family unsupported-optional-policy --max-edge 160 --output target/map-0152-unsupported-optional-policy.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/map-0152-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/map-rendering-manifest.tsv --include-family map --include-family raster-tile --include-family zoning-overlay --include-family optional-layer --include-family vector-layer --include-family pattern-layer --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/map-0152-visual-diff.json
cargo test -p pdfrust-native chart_dashboard -- --nocapture
cargo test -p pdfrust-native optional_content -- --nocapture
cargo test -p pdfrust-render rasterize_images -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/map-raster-tile-routes.pdf fixtures/generated/map-transparent-zoning-overlay.pdf fixtures/generated/map-optional-layer-policy.pdf
rg -n "private|customer|confidential|personal|production|PII|@" fixtures/corpus-manifest.tsv fixtures/map-rendering-manifest.tsv scripts/generate_fixtures.py
```
