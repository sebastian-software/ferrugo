# Shading Pattern Fidelity 2026-06-25

Milestone: 0090.

## Implemented Slice

- Added a deterministic generated fixture for unsupported mesh shading.
- Classified the mesh fixture as `unsupported` with feature bucket
  `graphics.pattern-shading`.
- Adjusted visual-diff classification so full-field low-amplitude gradient
  drift is accepted when the maximum channel delta stays within a strict
  bound.
- Kept high-delta field differences as visual-diff blockers.
- Verified existing axial gradient, radial gradient, and tiling pattern
  fixtures against PDFium.

## Fidelity Policy

| Feature | Current native behavior | Visual status |
| --- | --- | --- |
| Axial shading | Rendered natively with bounded color drift. | accepted drift |
| Radial shading | Rendered natively with bounded color drift. | accepted drift |
| Tiling pattern | Rendered natively with deterministic placement. | exact |
| Mesh shading | Explicit unsupported feature bucket. | native error |

The low-amplitude drift rule is intentionally narrow: it only applies when
mean absolute error stays within the existing threshold and every changed
channel remains at or below a maximum delta of `8`. This prevents gradient
rounding noise from hiding larger visual mismatches.

## Corpus Classification

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/shading-pattern-summary-0090.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 65 | 59 | 5 | 1 |

Fallback categories:

| Feature bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |

Report family summary:

| Total | Native rendered | Fallback required | Native pass rate |
| ---: | ---: | ---: | ---: |
| 13 | 12 | 1 | `0.923` |

The new report-family fallback is intentionally
`mesh-shading-unsupported.pdf`. The remaining error is the existing encrypted
fixture.

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/shading-pattern-visual-diff-0090.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 65 | 24 | 11 | 24 | 5 | 0 | 1 |

Report family summary:

| Total | Exact | Accepted drift | Blockers | Native errors |
| ---: | ---: | ---: | ---: | ---: |
| 13 | 6 | 4 | 2 | 1 |

Shading and pattern details:

| Fixture | Status | Changed ratio | MAE | P95 delta | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `axial-gradient.pdf` | accepted drift | `1.000000` | `1.328` | `3` | `3` |
| `radial-gradient.pdf` | accepted drift | `0.900556` | `1.236` | `4` | `5` |
| `tiling-pattern.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `mesh-shading-unsupported.pdf` | native error | n/a | n/a | n/a | n/a |

The mesh fixture fails natively with unsupported feature bucket
`graphics.pattern-shading`; PDFium renders it, so the visual-diff entry is a
native-only error rather than a blocker image.

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/shading-pattern-benchmark-0090.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 65 | 59 | 5 | 1 | 7 |

Report family summary:

| Total | Native rendered | Fallback required | Errors | Budget failures | Mean ms | Max ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 13 | 12 | 1 | 0 | 2 | `264.761` | `2835.054` |

Shading and pattern fixture outcomes:

| Fixture | Outcome | Reason | Mean ms |
| --- | --- | --- | ---: |
| `axial-gradient.pdf` | native rendered | n/a | `5.718` |
| `radial-gradient.pdf` | native rendered | n/a | `5.834` |
| `tiling-pattern.pdf` | native rendered | n/a | `31.724` |
| `mesh-shading-unsupported.pdf` | fallback required | `graphics.pattern-shading` | `0.037` |

The new budget failure is the intentional native fallback for the mesh-shading
fixture. It does not allocate a pattern cache or raster surface before
classification.

## Validation

```text
cargo fmt --check
cargo test -p pdfrust-cli visual_diff_metrics_should_accept_low_amplitude_field_drift
cargo test -p pdfrust-native mesh_shading
cargo test -p pdfrust-render shading
cargo test -p pdfrust-render pattern
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0090 touched files and passed.

## Remaining Limits

- Mesh shading remains a PDFium-fallback feature.
- Pattern cache policy is still limited to the current bounded renderer path;
  this slice did not add reusable pattern raster caches.
- Advanced mesh tessellation is planned separately in milestone 0108.
