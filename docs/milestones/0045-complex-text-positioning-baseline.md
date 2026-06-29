# 0045: Complex Text Positioning Baseline

Status: done
Phase: 5
Size: medium
Depends on: 0044

## Goal

Improve positioned text rendering for common kerning, spacing, and multi-run
PDF output without taking on full shaping.

## Scope

- Apply `TJ` spacing adjustments, character spacing, word spacing, and
  horizontal scaling consistently.
- Preserve text rendering modes that affect fill, stroke, and invisibility.
- Add reduced fixtures for office exports with fragmented text runs.
- Document where full shaping or vertical writing remains unsupported.

## Non-Goals

- Complex script shaping.
- Text selection or extraction APIs.
- Perfect printer-grade typography.

## Deliverables

- Text positioning improvements.
- Fixtures for fragmented and adjusted text runs.
- Updated unsupported text-feature documentation.

## Acceptance Criteria

- Common office/browser text thumbnails retain recognizable word spacing.
- Invisible and unsupported text modes do not corrupt rendering state.
- Unsupported shaping cases are explicit and measurable.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run PDFium differential pixel comparisons for text-heavy fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Implemented `Tc`, `Tw`, `Tz`, and `Tr` text-state handling in the native text
  display-list path.
- `TJ` arrays now emit separately positioned chunks around numeric adjustments,
  preserving intra-run spacing instead of applying adjustments only after the
  combined text.
- Text display items carry per-glyph origins and rendering mode metadata for
  the fallback rasterizer and later real glyph-outline rasterization.
- Added `fixtures/generated/text-spacing.pdf`, covering fragmented text,
  character/word spacing, horizontal scaling, and invisible text mode.
- Complex shaping, vertical writing, and text extraction remain unsupported
  non-goals.
- Validation: `cargo fmt --check`, `cargo check`, `cargo test`,
  `cargo test -p ferrugo-render -p ferrugo-native`,
  `cargo clippy --all-targets --all-features -- -D warnings`, native CLI smoke
  for `text-spacing.pdf`, and PDFium/native pixel comparison for
  `text-spacing.pdf` at `260x120`, MAE `13.265`, max `255`,
  `native_nonwhite_pixels=2284`.

## Progress Notes

- Added text-state handling for character spacing (`Tc`), word spacing (`Tw`),
  horizontal scaling (`Tz`), and text rendering mode (`Tr`).
- Changed `TJ` arrays to emit positioned chunks around numeric adjustments
  instead of applying all adjustments after the combined text.
- Carried per-glyph origins in text display items so spacing survives into the
  fallback rasterizer and later real glyph-outline rasterization.
