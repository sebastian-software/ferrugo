# Image Mask Stencil Coverage 2026-06-25

Milestone: 0088.

## Implemented Slice

- Added Image XObject stencil-mask mode for `/ImageMask true`.
- Applied stencil pixels with the current fill color from `g` and `rg`.
- Supported one-bit packed mask samples without expanding them to per-pixel
  alpha buffers.
- Supported `/Decode [0 1]` and `/Decode [1 0]` polarity, matching PDFium
  behavior for the generated corpus.
- Preserved `max_image_bytes` enforcement for packed mask streams.
- Added generated fixtures for signature, monochrome icon, and compressed logo
  stencil cases.

## Mask Fixtures

| Fixture | Purpose | Visual status |
| --- | --- | --- |
| `image-mask-logo.pdf` | Flate-compressed logo-style stencil. | exact |
| `image-mask-monochrome-icon.pdf` | One-bit monochrome icon stencil. | exact |
| `image-mask-signature.pdf` | Signature-style stencil over a line. | accepted drift |

The signature fixture drift is one-channel color rounding only:
`changed_ratio=0.020000`, `mean_abs_error=0.020`, `p95_channel_delta=0`, and
`max_channel_delta=1`.

## Memory Policy

ImageMask samples remain in the decoded one-bit row-packed representation. The
rasterizer samples bits directly with MSB-first row addressing, so this slice
does not allocate an expanded mask plane. The same `max_image_bytes` budget used
for other decoded image streams now gates ImageMask streams as well.

## Fallback Summary

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/image-mask-summary-0088.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 61 | 59 | 1 | 1 |

The remaining fallback is the existing optional-content membership policy. The
remaining error is the existing encrypted fixture. The `scan` and `form`
families both rendered 100% natively after adding the ImageMask fixtures.

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/image-mask-visual-diff-0088.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 61 | 24 | 9 | 26 | 1 | 0 | 1 |

ImageMask details:

| Fixture | Status | Changed ratio | MAE | P95 delta | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `image-mask-logo.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `image-mask-monochrome-icon.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `image-mask-signature.pdf` | accepted drift | `0.020000` | `0.020` | `0` | `1` |

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/image-mask-benchmark-0088.json
```

ImageMask fixture results:

| Fixture | Mean ms | Output bytes | Budget violations |
| --- | ---: | ---: | --- |
| `image-mask-logo.pdf` | `11.151` | `60000` | none |
| `image-mask-monochrome-icon.pdf` | `10.744` | `57600` | none |
| `image-mask-signature.pdf` | `10.530` | `38400` | none |

Corpus summary remained at 61 total fixtures, 59 native renders, 1 fallback, 1
error, and 3 known budget failures.

## Validation

```text
cargo fmt --check
cargo test -p pdfrust-render image_mask
cargo test -p pdfrust-native image_mask
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0088 touched files and passed.

## Remaining Limits

- Inline image masks are not implemented in this slice.
- ImageMask color handling currently tracks `DeviceGray` and `DeviceRGB` fill
  operators used by the image display-list path.
- Additional color-space and pattern-color stencils remain future work.
