# 0044: Glyph Outline Extraction

Status: done
Phase: 5
Size: medium
Depends on: 0043

## Goal

Extract glyph outlines for the supported font programs and convert them into
renderer path data.

## Scope

- Choose and document the outline extraction dependency or internal parser.
- Convert TrueType and CFF outlines into the existing path representation.
- Preserve glyph metrics required for thumbnail placement.
- Add cache boundaries for decoded glyph outlines.

## Non-Goals

- Advanced hinting.
- Subpixel text rendering.
- Full OpenType layout.

## Deliverables

- Glyph outline extraction layer.
- Glyph outline cache with memory limits.
- Unit tests for contour conversion and missing-glyph handling.

## Acceptance Criteria

- Supported fonts produce path data for visible glyphs.
- Missing glyphs use documented fallback behavior.
- Repeated glyphs reuse cached outlines instead of reparsing font data.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for text fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Implemented bounded glyph-outline extraction for embedded TrueType/SFNT and
  raw `/FontFile3` CFF programs through `ttf-parser`.
- Added a small glyph-outline cache with explicit entry and segment budgets.
- CFF extraction uses synthetic required OpenType tables around the raw CFF
  table, avoiding an additional parser dependency while keeping outline parsing
  in `ttf-parser`.
- Type1 `/FontFile` outlines remain unsupported and are tracked separately from
  the 0044 TrueType/CFF scope.
- Validation: `cargo fmt --check`, `cargo check`, `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`, native CLI
  smokes for `embedded-font.pdf`, `tounicode-text.pdf`, and
  `encoding-differences.pdf`, plus PDFium/native text comparison for
  `text-page.pdf` at `300x160`, MAE `12.082`, max `255`,
  `native_nonwhite_pixels=2653`.

## Progress Notes

- Selected `ttf-parser` 0.25.1 with only the `std` feature as the safe,
  zero-allocation outline parser dependency for SFNT-backed TrueType and CFF
  outlines.
- Added the initial glyph-outline API, metrics capture, path-segment
  conversion, segment budget, and cache boundaries.
- Added direct raw PDF CFF (`/FontFile3`) outline extraction by passing the CFF
  table to `ttf-parser::Face::from_raw_tables` with synthetic required
  OpenType tables.
