# AcroForm Widget Synthesis 2026-06-25

Milestone: 0092.

## Implemented Slice

- Added static native synthesis for missing-appearance `/Widget` annotations
  with `/FT /Tx`, `/FT /Ch`, and `/FT /Btn`.
- Rendered text and choice field values when the page exposes font resource
  `/F1`.
- Rendered checkbox and radio on-states from `/AS` or `/V`.
- Kept existing `/AP /N` appearances authoritative.
- Kept generated appearances isolated to transient synthetic content; no PDF
  objects are mutated or persisted.
- Added generated fixtures for text field, choice field, checkbox, and radio
  widgets without appearance streams.

## Policy

`docs/policies/acroform-appearances.md` was updated for 0092. JavaScript, XFA,
form editing, saving, calculations, and full viewer-specific widget styling
remain non-goals.

## Fallback Summary

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/acroform-synthesis-summary-0092.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 73 | 67 | 5 | 1 |

Form family:

| Total | Native rendered | Fallback required | Native pass rate | Errors |
| ---: | ---: | ---: | ---: | ---: |
| 11 | 11 | 0 | `1.000` | 0 |

Fallback categories:

| Feature bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/acroform-synthesis-visual-diff-0092.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 73 | 26 | 13 | 28 | 5 | 0 | 1 |

Form family:

| Total | Exact | Accepted drift | Blockers | Native errors |
| ---: | ---: | ---: | ---: | ---: |
| 11 | 0 | 1 | 10 | 0 |

New fixture details:

| Fixture | Status | Changed ratio | MAE | P95 delta | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `acroform-checkbox-missing-appearance.pdf` | blocker | `0.034688` | `5.706` | `0` | `255` |
| `acroform-choice-missing-appearance.pdf` | blocker | `0.175536` | `10.690` | `127` | `255` |
| `acroform-radio-missing-appearance.pdf` | blocker | `0.019500` | `4.014` | `0` | `255` |
| `acroform-text-field-missing-appearance.pdf` | blocker | `0.154107` | `8.452` | `20` | `255` |

The new widgets are visible and deterministic, but viewer-style parity remains
future work. PDFium synthesizes producer/viewer-specific widget styles that this
slice intentionally does not copy.

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/acroform-synthesis-benchmark-0092.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 73 | 67 | 5 | 1 | 7 |

Form family:

| Total | Native rendered | Fallback required | Errors | Budget failures | Mean ms | Max ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 11 | 0 | 0 | 0 | `25.417` | `81.366` |

New fixture outcomes:

| Fixture | Outcome | Mean ms | Budget violations |
| --- | --- | ---: | --- |
| `acroform-checkbox-missing-appearance.pdf` | native rendered | `20.989` | none |
| `acroform-choice-missing-appearance.pdf` | native rendered | `30.286` | none |
| `acroform-radio-missing-appearance.pdf` | native rendered | `81.366` | none |
| `acroform-text-field-missing-appearance.pdf` | native rendered | `30.675` | none |

## Validation

```text
cargo fmt --check
cargo test -p pdfrust-native acroform -- --nocapture
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0092 touched files and passed.

## Remaining Limits

- Widget styling is intentionally simple and still visually differs from PDFium
  synthesized styles.
- Text fallback currently depends on page font resource `/F1`; AcroForm `/DR`
  font inheritance is future work.
- Parent field inheritance, JavaScript-calculated values, XFA, popup UI, and
  editing behavior remain out of scope.
