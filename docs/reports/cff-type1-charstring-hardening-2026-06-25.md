# CFF Type1 Charstring Hardening

Date: 2026-06-25
Milestone: 0102

## Scope

This pass hardened glyph-outline handling for compact font programs and added a
bounded Type1 charstring subset path. The implementation keeps unsafe code out
of the font path and returns typed renderer errors for malformed or unsupported
charstring behavior.

## Implementation

- Added charstring stack and subroutine-depth limits to `GlyphOutlineOptions`.
- Added typed glyph-outline errors for charstring stack overflow and subroutine
  recursion overflow.
- Added a bounded Type1 charstring subset interpreter for synthetic FontFile
  programs, covering `hsbw`, `sbw`, `rmoveto`, `hmoveto`, `vmoveto`, `rlineto`,
  `hlineto`, `vlineto`, `rrcurveto`, `closepath`, `endchar`, and `div`.
- Kept unsupported Type1 operators typed through `UnsupportedGlyphOutline`
  rather than panicking or silently substituting outlines.
- Preserved the existing CFF path through bounded `ttf-parser` extraction.
- Added native bucket mapping for new charstring outline limit errors.

## Fixture Coverage

Added generated fixtures:

| Fixture | Family | Coverage |
| --- | --- | --- |
| `fixtures/generated/type1-fontfile-text.pdf` | `office-export` | Type1 FontFile text fixture with a bounded synthetic charstring program. |
| `fixtures/generated/cff-fontfile3-text.pdf` | `office-export` | Type1 resource with embedded CFF FontFile3 program. |

Unit tests cover:

- Successful Type1 charstring outline extraction.
- Malformed odd-length Type1 hex charstring rejection.
- Type1 charstring stack overflow.
- Type1 subroutine rejection through the configured recursion limit.
- Existing CFF outline extraction through the bounded CFF path.

Native smoke tests assert that both generated fixtures render without PDFium
fallback.

## Corpus Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/cff-type1-0102-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 6 | 6 | 0 | 0 |
| `office-export` | 17 | 17 | 0 | 0 |
| `form` | 12 | 12 | 0 | 0 |
| **Supported gate total** | **35** | **35** | **0** | **0** |

## PDFium Visual Comparison

Command:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/cff-type1-0102-visual-diff.json
```

Full-corpus summary after adding the fixtures:

| Metric | Count |
| --- | ---: |
| Total fixtures | 80 |
| Exact | 26 |
| Accepted drift | 13 |
| Blockers | 35 |
| Native errors | 5 |
| PDFium errors | 0 |
| Both errors | 1 |

New CFF/Type1 fixture comparison:

| Fixture | Status | Mean absolute error | Changed ratio | P95 delta |
| --- | --- | ---: | ---: | ---: |
| `cff-fontfile3-text.pdf` | blocker | 12.764 | 0.068611 | 136 |
| `type1-fontfile-text.pdf` | blocker | 13.368 | 0.072500 | 140 |

The new fixtures meet the native no-fallback requirement, but visual drift
remains blocked by the current built-in text rasterizer. This milestone hardens
font-program execution and typed failure behavior; visual parity remains part of
the later text/font fidelity milestones.

## Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo test -p pdfrust-render --no-default-features`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/cff-type1-0102-supported-gate.json`
- `cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --output target/cff-type1-0102-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
