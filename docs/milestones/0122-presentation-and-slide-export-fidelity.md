# 0122: Presentation And Slide Export Fidelity

Status: todo
Phase: 22
Size: medium
Depends on: 0121

## Goal

Improve native rendering for PDFs exported from slide tools, where gradients,
images, transparency, and positioned text commonly combine on one page.

## Scope

- Add fixtures for keynote-style slides, PowerPoint exports, and title decks.
- Cover layered images, soft shadows, gradients, rotated text, and speaker
  notes pages when visually relevant.
- Track accepted drift separately from hard blockers.
- Profile raster cost for full-bleed image and transparency-heavy slides.

## Non-Goals

- Parse original presentation formats.
- Preserve slide animations.
- Implement editing or speaker-note extraction APIs.

## Deliverables

- Slide-export fixture set.
- Fidelity report for presentation-specific visual features.
- Performance notes for transparency-heavy pages.

## Acceptance Criteria

- Common slide exports render without PDFium fallback.
- Transparency and gradient drift is classified with explicit thresholds.
- Large image slides stay within memory and output-size budgets.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run slide-export visual-diff comparisons.
- Run slide-export native benchmark.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
