# Visual Diff Dashboard 2026-06-24

Milestone: 0084.

## Implemented Slice

- Added `pdfrust-cli visual-diff` behind the optional `pdfium` feature.
- Compared Rust-native RGBA output against PDFium RGBA output through the shared
  thumbnail facade.
- Added per-fixture visual metrics:
  - changed pixels
  - changed ratio
  - mean absolute RGB error
  - p95 RGB channel delta
  - max RGB channel delta
  - non-white pixel counts for each backend
- Grouped output by corpus family and renderer subsystem.
- Classified fixtures as `exact`, `accepted_drift`, `blocker`,
  `native_error`, `pdfium_error`, or `both_error`.

## Command

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --max-edge 120 --output target/0084-visual-diff.json
```

## Summary

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 52 | 21 | 5 | 24 | 1 | 0 | 1 |

Default thresholds:

| Metric | Value |
| --- | ---: |
| max mean absolute error | 2.000 |
| max p95 channel delta | 16 |
| max changed ratio | 0.050000 |

## Subsystem Review

| Subsystem | Total | Exact | Accepted drift | Blockers | Native errors | Both errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| annotations-forms | 10 | 3 | 1 | 6 | 0 | 0 |
| document-security | 1 | 0 | 0 | 0 | 0 | 1 |
| document-structure | 3 | 2 | 0 | 1 | 0 | 0 |
| images-color | 8 | 6 | 0 | 2 | 0 | 0 |
| optional-content | 3 | 2 | 0 | 0 | 1 | 0 |
| page-geometry | 3 | 1 | 1 | 1 | 0 | 0 |
| rendering-core | 3 | 2 | 0 | 1 | 0 | 0 |
| text-fonts | 9 | 0 | 0 | 9 | 0 | 0 |
| transparency | 3 | 2 | 0 | 1 | 0 | 0 |
| vector-graphics | 9 | 3 | 3 | 3 | 0 | 0 |

## Family Review

| Family | Total | Exact | Accepted drift | Blockers | Native errors | Both errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| browser-export | 1 | 0 | 1 | 0 | 0 | 0 |
| form | 1 | 0 | 0 | 1 | 0 | 0 |
| invoice | 1 | 0 | 0 | 1 | 0 | 0 |
| malformed-recovery | 1 | 1 | 0 | 0 | 0 | 0 |
| office-export | 1 | 0 | 0 | 1 | 0 | 0 |
| presentation | 1 | 0 | 0 | 0 | 1 | 0 |
| report | 1 | 0 | 0 | 1 | 0 | 0 |
| scanned-packet | 1 | 0 | 0 | 1 | 0 | 0 |
| secure-document | 1 | 0 | 0 | 0 | 0 | 1 |
| statement | 1 | 0 | 0 | 1 | 0 | 0 |
| unclassified | 42 | 20 | 4 | 18 | 0 | 0 |

## Blocker Triage

Primary blocker buckets:

- `text-fonts`: 9 blockers. This is the largest user-visible gap and should
  continue through font, CMap, shaping, and glyph fidelity milestones.
- `annotations-forms`: 6 blockers. The native renderer draws appearances that
  PDFium omits or rasterizes differently in several generated form fixtures.
- `vector-graphics`: 3 blockers. Gradient and vector-stress fixtures exceed the
  default changed-ratio threshold.
- `images-color`: 2 blockers. CMYK and mixed text/image output need color and
  image-path review.
- `optional-content`: 1 native error. `optional-content-ocmd.pdf` remains a
  known unsupported native feature.
- `document-security`: 1 shared encrypted error. This is not visual drift.

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo test -p pdfrust-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

## Remaining Limits

- The report is JSON-only. It intentionally avoids committing rendered diff
  images until an artifact-retention policy exists.
- Subsystem assignment is heuristic and intended for triage, not release
  classification.
- The run is local-machine evidence against the current local PDFium dynamic
  library.
