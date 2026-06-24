# 0092: AcroForm Field Synthesis For Common Widgets

Status: completed
Phase: 15
Size: medium
Depends on: 0091

## Goal

Render common AcroForm widgets even when producer PDFs omit usable appearance
streams.

## Scope

- Synthesize appearances for text fields, checkboxes, radio buttons, and simple
  choice fields.
- Respect field values, default appearance hints, widget rectangles, and basic
  border/background styling.
- Keep generated appearances isolated from document mutation.
- Add fixtures from common fillable form producers.

## Non-Goals

- Implement JavaScript actions.
- Edit or save form values.
- Match every viewer-specific widget style.

## Deliverables

- Common widget appearance synthesis.
- Form policy and unsupported feature notes.
- Differential form fixture report.

## Acceptance Criteria

- Missing form appearances no longer block common form thumbnails.
- Synthesized widgets are deterministic and bounded by widget rectangles.
- JavaScript and dynamic XFA remain explicit non-goals.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run form visual comparisons.
- Run malformed form budget tests.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Completed 2026-06-25.
- Added bounded native synthesis for missing-appearance text, choice,
  checkbox, and radio widgets.
- Reused the existing annotation fallback content path and rendered optional
  widget text through page font resource `/F1` when available.
- Kept JavaScript, XFA, editing, saving, and full viewer-style parity out of
  scope.
- Added deterministic missing-appearance AcroForm fixtures.
- Updated `docs/policies/acroform-appearances.md`.
- Recorded evidence in
  `docs/reports/acroform-widget-synthesis-2026-06-25.md`.
