# OpenType Layout Feature Coverage

Date: 2026-06-25
Milestone: 0103

## Summary

Milestone 0103 adds a bounded native fallback layer for PDF text that already
arrives as shaped or glyph-expanded output. The native renderer now records a
typed layout status per decoded PDF glyph, expands common multi-character
ToUnicode mappings such as ligatures during fallback rasterization, positions
combining marks over the previous base glyph, and preserves pre-positioned
Arabic/Hebrew-style script output as explicit shaped text.

This is not a full OpenType GSUB/GPOS implementation. Full embedded font table
shaping remains a later text/font fidelity task. The current slice makes common
PDF-exported shaped output visible and diagnosable without PDFium fallback.

## Implementation

- Added `TextLayoutStatus` and `TextLayoutFallbackReason` to decoded
  `TextGlyph` metadata.
- Classified decoded text into simple glyphs, ligature expansions, combining
  mark positioning, pre-shaped script preservation, and typed unsupported
  complex-script fallback.
- Added reusable `TextRasterScratch` state for expanded fallback text atoms so
  repeated rasterization reuses capacity instead of allocating per glyph.
- Updated fallback text rasterization to draw all Unicode scalars mapped from
  one PDF source glyph instead of only the first scalar.
- Added a minimal visible combining-mark fallback above the previous base
  glyph.
- Added generated fixtures:
  - `fixtures/generated/opentype-ligature-text.pdf`
  - `fixtures/generated/combining-mark-text.pdf`
  - `fixtures/generated/arabic-shaped-text.pdf`

## Evidence

Supported-family native-only gate:

- Total: 38
- Native rendered: 38
- Fallback required: 0
- Errors: 0
- Browser-print: 6/6 native rendered
- Form: 12/12 native rendered
- Office-export: 20/20 native rendered
- Artifact: `target/opentype-layout-0103-supported-gate.json`

PDFium visual comparison:

- Total: 83
- Exact: 28
- Accepted drift: 6
- Blockers: 43
- Native errors: 5
- PDFium errors: 0
- Both errors: 1
- Artifact: `target/opentype-layout-0103-visual-diff.json`

New shaped-text fixtures render natively without errors, but remain visual
blockers against PDFium because the current built-in fallback text rasterizer is
not font-shape or glyph-metric equivalent to PDFium:

| Fixture | Status | MAE | Changed Ratio | p95 |
| --- | --- | ---: | ---: | ---: |
| `arabic-shaped-text.pdf` | blocker | 11.411 | 0.055267 | 38 |
| `combining-mark-text.pdf` | blocker | 6.039 | 0.029143 | 0 |
| `opentype-ligature-text.pdf` | blocker | 9.048 | 0.039396 | 0 |

## Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p ferrugo-render`
- `cargo test -p ferrugo-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/opentype-layout-0103-supported-gate.json`
- `FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 1.0 --max-p95 8 --max-changed-ratio 0.02 --output target/opentype-layout-0103-visual-diff.json`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Follow-Ups

- Implement real GSUB/GPOS table shaping for embedded OpenType fonts.
- Improve fallback glyph metrics so ligature and mark fixtures can move from
  visual blockers to accepted drift.
- Add script-specific fixtures once the shaping layer can prove more than
  pre-positioned output preservation.
