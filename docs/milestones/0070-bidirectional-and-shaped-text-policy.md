# 0070: Bidirectional And Shaped Text Policy

Status: todo
Phase: 11
Size: medium
Depends on: 0069

## Goal

Define and implement the native renderer policy for bidirectional and shaped
text as it appears in PDFs.

## Scope

- Measure how typical PDFs encode Arabic, Hebrew, Indic, and shaped Latin text.
- Decide when PDF glyph positioning is sufficient and when shaping support is
  required.
- Add targeted support or explicit unsupported categories for observed cases.
- Keep text rendering deterministic and allocation-aware.

## Non-Goals

- Shape arbitrary Unicode source text before PDF layout.
- Build a full text extraction or accessibility layer.
- Add heavyweight text dependencies without benchmark evidence.

## Deliverables

- Text shaping decision record.
- Fixtures for shaped and bidirectional documents.
- Renderer support or clear fallback policy for each observed category.

## Acceptance Criteria

- Typical pre-shaped PDF text renders in native mode where glyph data is present.
- Unsupported shaped-text cases are typed and covered by tests.
- Dependency and memory tradeoffs are documented.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run shaped-text fixture comparisons.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
