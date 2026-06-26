# 0169: Font Fallback Script Mixing And Emoji Coverage

Status: done
Phase: 31
Size: medium
Depends on: 0168

## Goal

Improve rendering of typical documents that mix Latin text with CJK, RTL,
symbols, math glyphs, and emoji fallback fonts.

## Scope

- Add fixtures for mixed-script office, browser, form, and chat-export PDFs.
- Audit font fallback selection, glyph cache keys, color glyph policy, and
  missing-glyph diagnostics.
- Implement bounded improvements for common fallback sequences.
- Keep unsupported color-font or shaping behavior explicit when not implemented.

## Non-Goals

- Build complete text shaping for every script in this milestone.
- Guarantee pixel-identical emoji rendering across platforms.
- Vendor large system font bundles into the package.

## Deliverables

- Mixed-script and emoji fixture set.
- Font fallback coverage report.
- Renderer fixes for accepted fallback cases.

## Acceptance Criteria

- Common mixed-script documents avoid missing-glyph regressions.
- Fallback selection is deterministic enough for supported platforms.
- Unsupported font behavior returns typed diagnostics.

## Validation

- Run native-only `cargo test`.
- Run mixed-script visual comparison subset.
- Run glyph cache benchmark subset.
- Run platform determinism checks where available.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

Report:

- `docs/reports/font-fallback-script-mixing-coverage-2026-06-26.md`

Implemented:

- Added `fixtures/font-fallback-script-mixing-manifest.tsv` for CJK, RTL,
  ligature/combining, missing-font, symbol, and emoji-boundary coverage.
- Added `chat-emoji-fallback-boundary.pdf` as a typed unsupported boundary for
  chat-export emoji/color-font behavior.
- Added renderer and native tests for emoji layout classification and the
  generated typed unsupported boundary.

Validation:

- `cargo test -p pdfrust-render emoji -- --nocapture`
- `cargo test -p pdfrust-native emoji -- --nocapture`
- `cargo test -p pdfrust-render glyph_bitmap_cache -- --nocapture`
- `cargo test -p pdfrust-render font_resources_should_bound_fallback_resolution_cache -- --nocapture`
- `cargo test -p pdfrust-render font_resources_should_resolve_missing_embedded_font_deterministically -- --nocapture`
- Supported mixed-script native gate: 12 total, 12 native rendered, 0 fallbacks,
  0 errors.
- Full mixed-script boundary summary: emoji boundary returns 1
  `text.font-program` fallback; all other families render natively.
- Mixed-script benchmark: 12 total, 12 native rendered, 0 fallbacks, 0 errors,
  0 budget failures.
- Maintainer visual comparison: 12 total, 0 exact, 1 accepted drift, 11
  blockers, 0 native errors, 0 PDFium errors.
