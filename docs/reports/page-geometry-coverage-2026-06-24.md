# Page Geometry Coverage 2026-06-24

Milestone: 0085.

## Implemented Slice

- Parsed inherited `/Rotate` page-tree metadata and normalized it to `0`, `90`,
  `180`, or `270` degrees.
- Parsed and bounded `/UserUnit` values.
- Wired native page geometry into the existing raster transform so rotation and
  CropBox selection affect output dimensions.
- Added generated fixtures for rotated office exports, cropped scans, and
  non-default UserUnit pages.
- Kept page raster dimensions bounded by existing `max_edge` and page pixel
  budgets.

## UserUnit Policy

The native renderer parses and validates `/UserUnit`, but the thumbnail geometry
path keeps PDFium-compatible page dimensions for this API surface. Local PDFium
evidence rendered `user-unit-page.pdf` as `80 x 60`, matching the unscaled
MediaBox dimensions, so native thumbnail rendering follows that behavior for
now.

## Geometry Fixtures

| Fixture | Purpose | Native/PDFium dimensions | Status |
| --- | --- | --- | --- |
| `cropped-scan-page.pdf` | CropBox page selection for scan-like content. | `120 x 120` | exact |
| `rotated-office-export.pdf` | `/Rotate 90` office-style page. | `100 x 160` | blocker |
| `user-unit-page.pdf` | Non-default `/UserUnit 2`. | `80 x 60` | blocker |

The blockers are visual-fidelity differences, not page-size mismatches.

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/0085-geometry-visual-diff.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 55 | 22 | 5 | 26 | 1 | 0 | 1 |

Geometry fixture details:

| Fixture | Changed ratio | MAE | P95 delta | Notes |
| --- | ---: | ---: | ---: | --- |
| `cropped-scan-page.pdf` | `0.000000` | `0.000` | `0` | Exact CropBox match. |
| `rotated-office-export.pdf` | `0.193000` | `16.074` | `171` | Same dimensions; text/vector visual drift remains. |
| `user-unit-page.pdf` | `0.076875` | `13.125` | `139` | Same dimensions; text/vector visual drift remains. |

## Validation

```text
cargo fmt --check
cargo check
cargo test
cargo test -p pdfrust-cli --features pdfium
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0085 touched files and passed.

## Remaining Limits

- Rotated text and vector rendering still differs from PDFium and remains a
  visual-diff blocker.
- UserUnit rendering now matches PDFium dimensions for this API surface, but
  future full-spec work may need a separate policy if another consumer expects
  strict PDF UserUnit scaling.
