# Engineering Drawing Precision Gate

Date: 2026-06-26.
Milestone: 0151.

## Summary

The technical drawing corpus now includes three additional engineering-focused
reductions in `fixtures/technical-drawing-manifest.tsv`. They extend the older
CAD-style gate with floorplan, schematic, and large-coordinate transform
coverage.

New fixtures:

| Fixture | Coverage |
| --- | --- |
| `engineering-floorplan-precision.pdf` | Fine floorplan grid, thin wall strokes, dashed dimension line, doors, dimensions, and labels. |
| `engineering-schematic-symbols.pdf` | Schematic symbol sheet with repeated symbols, buses, dashed guide, and labels. |
| `engineering-large-transform-detail.pdf` | Large-coordinate engineering detail with dense grid, dimension marker, and downscaled linework. |

## Native Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --fail-on-fallback --max-edge 160 --output target/engineering-0151-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 11 | 11 | 0 | 0 |

## Vector Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/engineering-0151-benchmark.json
```

Result:

| Family | Total | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| `floorplan` | 1 | 82.876 | 82.876 | 0 |
| `hatch-clipping` | 2 | 174.209 | 301.596 | 0 |
| `large-coordinate` | 2 | 37.658 | 70.356 | 0 |
| `linework` | 2 | 32.978 | 64.829 | 0 |
| `repeated-symbols` | 2 | 125.844 | 190.361 | 0 |
| `schematic` | 1 | 24.088 | 24.088 | 0 |
| `transform-detail` | 1 | 82.141 | 82.141 | 0 |

## Visual Oracle

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/engineering-0151-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 2 | 0 | 9 | 0 | 0 |

Subsystem result:

| Subsystem | Total | Exact | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: |
| `page-geometry` | 1 | 0 | 1 | 0 |
| `rendering-core` | 7 | 0 | 7 | 0 |
| `vector-graphics` | 3 | 2 | 1 | 0 |

New fixture classifications:

| Fixture | Status | Subsystem | MAE | p95 | Changed ratio |
| --- | --- | --- | ---: | ---: | ---: |
| `engineering-floorplan-precision.pdf` | blocker | `rendering-core` | 5.680 | 38 | 0.301151 |
| `engineering-large-transform-detail.pdf` | blocker | `rendering-core` | 8.306 | 44 | 0.304556 |
| `engineering-schematic-symbols.pdf` | blocker | `rendering-core` | 5.513 | 27 | 0.099942 |

These blockers are precision/fidelity deltas, not native runtime fallbacks. The
exact `dashed-stroke.pdf` and `clipped-paths.pdf` rows remain useful control
baselines while linework, hatch, repeated-symbol, and large-coordinate pages
route to stroke placement and transform parity work.

## Size And Privacy

| Fixture | Bytes |
| --- | ---: |
| `engineering-floorplan-precision.pdf` | 1,683 |
| `engineering-schematic-symbols.pdf` | 1,447 |
| `engineering-large-transform-detail.pdf` | 1,565 |
| **Total new PDF bytes** | **4,695** |

Checks:

- `find fixtures/generated -name '*.pdf' -size +512k -print` returned no rows.
- `rg -n "private|customer|confidential|personal|production|PII|@" ...`
  returned only synthetic "no customer/no private data" fixture text plus an
  existing confidentiality clause in an unrelated contract fixture generator.
- New fixture content is synthetic and has no real CAD, floorplan, or facility
  drawing source.

## Validation

Commands run:

```sh
python3 scripts/generate_fixtures.py
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --fail-on-fallback --max-edge 160 --output target/engineering-0151-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/engineering-0151-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/technical-drawing-manifest.tsv --include-family linework --include-family hatch-clipping --include-family large-coordinate --include-family repeated-symbols --include-family floorplan --include-family schematic --include-family transform-detail --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/engineering-0151-visual-diff.json
cargo test -p pdfrust-native technical_drawing -- --nocapture
cargo test -p pdfrust-render rasterize_paths -- --nocapture
cargo test -p pdfrust-render page_transform -- --nocapture
find fixtures/generated -name '*.pdf' -size +512k -print
wc -c fixtures/generated/engineering-floorplan-precision.pdf fixtures/generated/engineering-schematic-symbols.pdf fixtures/generated/engineering-large-transform-detail.pdf
rg -n "private|customer|confidential|personal|production|PII|@" fixtures/corpus-manifest.tsv fixtures/technical-drawing-manifest.tsv scripts/generate_fixtures.py
```
