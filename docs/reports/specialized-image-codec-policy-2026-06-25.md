# Specialized Image Codec Policy 2026-06-25

Milestone: 0089.

## Implemented Slice

- Added PDF image filter alias handling for `/Fl` and `/DCT`.
- Kept CCITT, JPX, and JBIG2 as explicit deferred native codecs.
- Added `/CCF` alias coverage for CCITT fallback classification.
- Added deterministic generated fixtures for unsupported CCITT, JBIG2, and JPX
  image streams.
- Added native backend tests proving those fixtures map to `unsupported` with
  feature bucket `image.filter`.
- Recorded the strategy in
  `docs/decisions/0006-specialized-image-codec-policy.md`.

## Codec Strategy

| Filter | Native strategy | Current behavior |
| --- | --- | --- |
| `FlateDecode`, `Fl` | Existing safe stream decoder plus predictor support. | supported |
| `DCTDecode`, `DCT` | Existing safe Rust JPEG decoder path. | supported |
| `CCITTFaxDecode`, `CCF` | Defer until corpus-backed safe decoder selection. | `image.filter` fallback |
| `JPXDecode` | Defer until safe or isolated JPEG 2000 decoder selection. | `image.filter` fallback |
| `JBIG2Decode` | Defer until sandboxed or strongly isolated decoder strategy. | `image.filter` fallback |
| Other uncommon image filters | No silent fallback inside native rendering. | `UnsupportedImageFilter` |

## Corpus Classification

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/codec-policy-summary-0089.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Encrypted errors |
| ---: | ---: | ---: | ---: |
| 64 | 59 | 4 | 1 |

Fallback categories:

| Feature bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 1 |

Scan family summary:

| Total | Native rendered | Fallback required | Native pass rate |
| ---: | ---: | ---: | ---: |
| 13 | 10 | 3 | `0.769` |

The three scan fallbacks are intentionally generated codec-policy fixtures:
`unsupported-ccitt-image.pdf`, `unsupported-jbig2-image.pdf`, and
`unsupported-jpx-image.pdf`.

## Benchmark Run

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/codec-policy-benchmark-0089.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 64 | 59 | 4 | 1 | 6 |

Codec-policy fixture outcomes:

| Fixture | Outcome | Reason | Mean ms |
| --- | --- | --- | ---: |
| `unsupported-ccitt-image.pdf` | fallback required | `image.filter` | `0.093` |
| `unsupported-jbig2-image.pdf` | fallback required | `image.filter` | `0.040` |
| `unsupported-jpx-image.pdf` | fallback required | `image.filter` | `0.039` |

The three added budget failures are intentional `native_fallback` entries for
the codec-policy fixtures. No scan fixture reports a render error.

## Validation

```text
cargo fmt --check
cargo test -p ferrugo-render image_resources_should_decode_flate_alias_xobject
cargo test -p ferrugo-render image_resources_should_route_dct_alias_to_jpeg_decoder
cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs
cargo test -p ferrugo-native unsupported_
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0089 touched files and passed.

## Remaining Limits

- CCITT, JPX, and JBIG2 are still PDFium-fallback codecs.
- The generated codec-policy fixtures contain intentionally minimal payloads
  because the native renderer should reject these filters before attempting
  decoder-specific payload validation.
- A future implementation slice needs valid codec payload fixtures before
  claiming visual parity for any of these codecs.
