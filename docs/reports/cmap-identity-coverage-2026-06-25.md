# CMap Identity Coverage

Date: 2026-06-25
Milestone: 0104

## Summary

Milestone 0104 extends native text decoding for common CID-font CMap cases. The
renderer now parses `begincodespacerange` sections in ToUnicode CMaps, accepts
`Identity-H` and `Identity-V` as base `usecmap` entries, and creates a bounded
two-byte identity Unicode mapping for Type0 fonts that use `/Encoding
/Identity-H` or `/Identity-V` without a ToUnicode stream.

This is still a pragmatic renderer slice. It does not implement recursive named
CMap resource lookup or arbitrary `usecmap` include graphs. Non-identity
`usecmap` remains a typed unsupported CMap feature.

## Implementation

- Added code-space range tracking to `ToUnicodeMap`.
- Added longest-match lookup that respects parsed code-space ranges.
- Added dynamic two-byte identity mapping for Type0 Identity-H/V fonts without
  a ToUnicode stream.
- Allowed `/Identity-H usecmap` and `/Identity-V usecmap` as explicit base CMap
  references.
- Kept malformed code-space ranges deterministic through `InvalidCMap`.
- Added generated fixtures:
  - `fixtures/generated/cmap-codespace-range-text.pdf`
  - `fixtures/generated/identity-h-cjk-text.pdf`
  - `fixtures/generated/identity-v-cjk-text.pdf`

## Evidence

Supported-family native-only gate:

- Total: 41
- Native rendered: 41
- Fallback required: 0
- Errors: 0
- Browser-print: 6/6 native rendered
- Form: 12/12 native rendered
- Office-export: 23/23 native rendered
- Artifact: `target/cmap-0104-supported-gate.json`

PDFium visual comparison:

- Total: 86
- Exact: 28
- Accepted drift: 6
- Blockers: 46
- Native errors: 5
- PDFium errors: 0
- Both errors: 1
- Artifact: `target/cmap-0104-visual-diff.json`

New CMap fixtures render natively without fallback or native errors. They remain
visual blockers because the fallback text rasterizer draws deterministic
placeholder text while PDFium renders no visible glyphs for these synthetic CID
font programs:

| Fixture | Status | MAE | Changed Ratio | p95 |
| --- | --- | ---: | ---: | ---: |
| `cmap-codespace-range-text.pdf` | blocker | 6.160 | 0.024157 | 0 |
| `identity-h-cjk-text.pdf` | blocker | 6.160 | 0.024157 | 0 |
| `identity-v-cjk-text.pdf` | blocker | 4.826 | 0.018925 | 0 |

## Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p pdfrust-render`
- `cargo test -p pdfrust-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/cmap-0104-supported-gate.json`
- `PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 1.0 --max-p95 8 --max-changed-ratio 0.02 --output target/cmap-0104-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Follow-Ups

- Add bounded named CMap resource lookup before accepting arbitrary non-identity
  `usecmap`.
- Add cycle detection if recursive CMap resources become supported.
- Improve CID font program rendering so identity CMap fixtures can move from
  native-visible placeholders to PDFium visual parity.
