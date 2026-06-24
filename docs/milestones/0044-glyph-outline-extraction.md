# 0044: Glyph Outline Extraction

Status: in-progress
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

Empty until done.

## Progress Notes

- Selected `ttf-parser` 0.25.1 with only the `std` feature as the safe,
  zero-allocation outline parser dependency for SFNT-backed TrueType outlines.
- Added the initial glyph-outline API, metrics capture, path-segment
  conversion, segment budget, and cache boundaries.
- Raw PDF CFF (`/FontFile3`) outline extraction remains open for this
  milestone.
