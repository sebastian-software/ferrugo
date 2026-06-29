# Raster Image Heavy Memory Gate 2026-06-26

Milestone: 0170

## Summary

Added a focused image-heavy memory gate for the Rust-native renderer. The gate
covers repeated image XObject placement, rotated masked images, large scanner
pages, mixed Flate/JPEG scan content, soft masks, image masks, PNG predictor
decode, and DCT/JPEG images.

The renderer already keeps decoded image samples in `Arc<[u8]>` resources and
clones only the resource handle for repeated placements. No unbounded decoded
RGBA cache was added. The new coverage verifies that image-heavy documents stay
inside the existing native memory and output budgets in both default and
low-memory native profiles.

## Fixture Coverage

Added `fixtures/image-heavy-memory-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `repeated-xobject` | 1 | One Flate RGB image XObject reused across many placements. |
| `rotated-mask` | 1 | Rotated RGB image placements with a Flate soft mask. |
| `large-scan` | 1 | Existing large compressed scanner image budget fixture. |
| `mixed-compression` | 1 | Existing mobile scan fixture combining Flate and DCT images. |
| `soft-mask` | 1 | Existing image soft-mask alpha fixture. |
| `image-mask` | 1 | Existing compressed ImageMask logo fixture. |
| `predictor` | 1 | Existing Flate PNG predictor fixture. |
| `jpeg` | 1 | Existing DCTDecode Image XObject fixture. |

New generated fixtures:

- `fixtures/generated/image-heavy-repeated-xobject-report.pdf`
- `fixtures/generated/image-heavy-rotated-mask-sheet.pdf`

Both are included in the main corpus manifest with `expected:native`.

## Native Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/image-heavy-memory-manifest.tsv \
  --include-family repeated-xobject \
  --include-family rotated-mask \
  --include-family large-scan \
  --include-family mixed-compression \
  --include-family soft-mask \
  --include-family image-mask \
  --include-family predictor \
  --include-family jpeg \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/image-heavy-0170-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 8 | 8 | 0 | 0 |

## Benchmark And Memory Budget

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/image-heavy-memory-manifest.tsv \
  --include-family repeated-xobject \
  --include-family rotated-mask \
  --include-family large-scan \
  --include-family mixed-compression \
  --include-family soft-mask \
  --include-family image-mask \
  --include-family predictor \
  --include-family jpeg \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/image-heavy-0170-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `image-mask` | 1 | 1 | 0 | 0 | 0 | 11.902 | 11.902 | 60000 |
| `jpeg` | 1 | 1 | 0 | 0 | 0 | 1.600 | 1.600 | 57600 |
| `large-scan` | 1 | 1 | 0 | 0 | 0 | 31.697 | 31.697 | 74240 |
| `mixed-compression` | 1 | 1 | 0 | 0 | 0 | 28.793 | 28.793 | 71040 |
| `predictor` | 1 | 1 | 0 | 0 | 0 | 0.967 | 0.967 | 57600 |
| `repeated-xobject` | 1 | 1 | 0 | 0 | 0 | 49.669 | 49.669 | 85120 |
| `rotated-mask` | 1 | 1 | 0 | 0 | 0 | 46.967 | 46.967 | 74240 |
| `soft-mask` | 1 | 1 | 0 | 0 | 0 | 1.142 | 1.142 | 57600 |

Low-memory native profile command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/image-heavy-memory-manifest.tsv \
  --include-family repeated-xobject \
  --include-family rotated-mask \
  --include-family large-scan \
  --include-family mixed-compression \
  --include-family soft-mask \
  --include-family image-mask \
  --include-family predictor \
  --include-family jpeg \
  --native-profile low-memory \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/image-heavy-0170-low-memory-benchmark.json
```

Result: 8 total, 8 native rendered, 0 fallbacks, 0 errors, 0 budget failures.

## Visual Comparison

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/image-heavy-memory-manifest.tsv \
  --include-family repeated-xobject \
  --include-family rotated-mask \
  --include-family large-scan \
  --include-family mixed-compression \
  --include-family soft-mask \
  --include-family image-mask \
  --include-family predictor \
  --include-family jpeg \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.05 \
  --output target/image-heavy-0170-visual-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 4 | 0 | 4 | 0 | 0 |

Exact matches: `dct-image.pdf`, `image-mask-logo.pdf`, `predictor-image.pdf`,
and `soft-mask-image.pdf`.

Blockers are image resampling/interpolation fidelity follow-ups, not support or
memory failures:

| Fixture | Status | MAE | P95 channel delta | Changed ratio |
| --- | --- | ---: | ---: | ---: |
| `image-heavy-repeated-xobject-report.pdf` | blocker | 5.539 | 31 | 0.509398 |
| `image-heavy-rotated-mask-sheet.pdf` | blocker | 3.670 | 12 | 0.417457 |
| `mobile-mixed-compression-scan.pdf` | blocker | 4.273 | 15 | 0.933615 |
| `scanner-large-image-budget.pdf` | blocker | 2.079 | 8 | 0.239278 |

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo test -p ferrugo-native image_heavy -- --nocapture`
- Native supported gate, default benchmark, low-memory benchmark, and visual
  comparison commands listed above.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
