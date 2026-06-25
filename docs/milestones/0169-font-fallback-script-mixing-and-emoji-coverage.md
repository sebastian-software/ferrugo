# 0169: Font Fallback Script Mixing And Emoji Coverage

Status: todo
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

Empty until done.
