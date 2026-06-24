# Type3 Font Coverage 2026-06-25

Milestone: 0086.

## Implemented Slice

- Parsed Type3 font dictionaries, font matrices, font bounding boxes, encodings,
  explicit glyph widths, and referenced CharProc streams.
- Executed Type3 CharProc path content through the existing display-list
  interpreter and path rasterizer.
- Kept Type3 metadata behind `Arc` on font descriptors to avoid large text
  display items and repeated CharProc payload copies.
- Added deterministic generated fixtures for simple vector glyphs,
  symbol-like glyphs, and barcode-like glyphs.
- Added malformed/budget coverage for oversized CharProc streams.

## Type3 Fixtures

| Fixture | Purpose | Visual status |
| --- | --- | --- |
| `type3-vector-text.pdf` | Simple filled vector glyph CharProcs. | accepted drift |
| `type3-symbol-font.pdf` | Symbol-like colored vector glyph CharProc. | accepted drift |
| `type3-barcode-font.pdf` | Barcode-like vertical bar glyph CharProc. | accepted drift |

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/type3-visual-diff-0086.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 58 | 22 | 8 | 26 | 1 | 0 | 1 |

Type3 fixture details:

| Fixture | Changed ratio | MAE | P95 delta | Notes |
| --- | ---: | ---: | ---: | --- |
| `type3-vector-text.pdf` | `0.015302` | `0.440` | `0` | Accepted anti-aliasing drift. |
| `type3-symbol-font.pdf` | `0.031681` | `1.643` | `0` | Accepted anti-aliasing drift. |
| `type3-barcode-font.pdf` | `0.043265` | `1.526` | `0` | Accepted anti-aliasing drift. |

## Fallback Summary

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/type3-summary-0086.json
```

The generated corpus reported 58 fixtures total: 56 native renders, 1 optional
content fallback, and 1 encrypted error. The `office-export` family reported 14
of 14 native renders after adding the Type3 fixtures.

## Validation

```text
cargo fmt --check
cargo check --workspace --no-default-features
cargo test -p pdfrust-render type3
cargo test -p pdfrust-native type3
cargo test --workspace --no-default-features
cargo clippy -p pdfrust-render --no-default-features -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

`cargo clippy --workspace --no-default-features -- -D warnings` was also tried
and exposed existing CLI `needless_return` warnings in the no-default PDFium
disabled code path. That is outside the 0086 Type3 slice; the milestone's
all-features Clippy gate passed.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0086 touched files and passed.

## Remaining Limits

- Type3 CharProcs currently reuse the path/color subset already supported by
  the page display-list interpreter.
- Nested text/image-heavy CharProcs remain future work unless corpus evidence
  shows they matter for thumbnails.
