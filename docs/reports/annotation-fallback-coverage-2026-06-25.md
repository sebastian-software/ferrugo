# Annotation Fallback Coverage 2026-06-25

Milestone: 0091.

## Implemented Slice

- Added native fallback rendering for appearance-free `/Highlight`,
  `/Underline`, `/Square`, `/Circle`, and `/Text` annotations.
- Kept appearance streams authoritative when `/AP /N` is present.
- Kept appearance-free `/Link` annotations invisible.
- Added local synthetic ExtGState resources for annotation fallback drawing.
- Capped fallback QuadPoint processing at 32 quads per annotation.
- Used a 12-segment polygonal ellipse for `/Circle` to avoid expensive cubic
  fallback geometry.
- Added generated fixtures for highlight, link, review markup, and text note
  annotations without appearance streams.

## Policy

Fallback behavior is documented in `docs/policies/annotation-fallbacks.md`.
The renderer does not execute annotation actions, JavaScript, URIs, popups, or
external behavior while rendering thumbnails.

## Fallback Summary

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/annotation-fallback-summary-0091.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 69 | 63 | 5 | 1 |

Fallback categories:

| Feature bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |

Mixed-layout family:

| Total | Native rendered | Fallback required | Native pass rate | Errors |
| ---: | ---: | ---: | ---: | ---: |
| 13 | 12 | 0 | `0.923` | 1 |

The mixed-layout error is the existing encrypted fixture. The new
appearance-free annotation fixtures render natively.

## Visual-Diff Run

Command:

```text
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/annotation-fallback-visual-diff-0091.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 69 | 26 | 13 | 24 | 5 | 0 | 1 |

Annotation subsystem:

| Total | Exact | Accepted drift | Blockers | Native errors |
| ---: | ---: | ---: | ---: | ---: |
| 15 | 5 | 4 | 6 | 0 |

New fixture details:

| Fixture | Status | Changed ratio | MAE | P95 delta | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `highlight-annotation-without-appearance.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `link-annotation-without-appearance.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `markup-annotations-without-appearance.pdf` | accepted drift | `0.036528` | `1.996` | `0` | `255` |
| `text-note-annotation-without-appearance.pdf` | accepted drift | `0.013194` | `1.183` | `0` | `255` |

## Benchmark Run

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/annotation-fallback-benchmark-0091.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 69 | 63 | 5 | 1 | 7 |

Mixed-layout family:

| Total | Native rendered | Fallback required | Errors | Budget failures | Mean ms | Max ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 13 | 12 | 0 | 1 | 1 | `35.069` | `170.062` |

New fixture outcomes:

| Fixture | Outcome | Mean ms | Budget violations |
| --- | --- | ---: | --- |
| `highlight-annotation-without-appearance.pdf` | native rendered | `23.582` | none |
| `link-annotation-without-appearance.pdf` | native rendered | `15.648` | none |
| `markup-annotations-without-appearance.pdf` | native rendered | `170.062` | none |
| `text-note-annotation-without-appearance.pdf` | native rendered | `71.624` | none |

## Validation

```text
cargo fmt --check
cargo test -p ferrugo-native annotation -- --nocapture
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0091 touched files and passed.

## Remaining Limits

- Existing annotation blockers from earlier appearance rendering remain in the
  broader visual-diff dashboard.
- Text note icons are static thumbnail approximations; popup chrome and note
  contents are intentionally not rendered.
- Unknown annotation subtypes without appearances remain skipped.
