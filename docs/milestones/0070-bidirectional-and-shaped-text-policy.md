# 0070: Bidirectional And Shaped Text Policy

Status: done
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

Completed on 2026-06-24.

- Added decision record
  `docs/decisions/0004-bidirectional-and-shaped-text-policy.md`.
- Documented the renderer policy: respect pre-shaped PDF glyph codes and
  positioning; do not shape Unicode source text in this phase.
- Added `fixtures/generated/shaped-rtl-text.pdf`, a pre-positioned Hebrew
  Type0/CID fixture with ToUnicode mapping.
- Added native backend coverage for rendering the shaped RTL fixture without
  fallback.
- Recorded fallback and PDFium comparison results in
  `docs/reports/shaped-text-policy-coverage-2026-06-24.md`.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo test -p ferrugo-native`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - shaped-text fallback summary through the office-export family
  - native/PDFium render comparison for `shaped-rtl-text.pdf`
