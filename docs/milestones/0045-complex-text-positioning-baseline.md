# 0045: Complex Text Positioning Baseline

Status: in-progress
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

Empty until done.

## Progress Notes

- Added text-state handling for character spacing (`Tc`), word spacing (`Tw`),
  horizontal scaling (`Tz`), and text rendering mode (`Tr`).
- Changed `TJ` arrays to emit positioned chunks around numeric adjustments
  instead of applying all adjustments after the combined text.
- Carried per-glyph origins in text display items so spacing survives into the
  fallback rasterizer and later real glyph-outline rasterization.
