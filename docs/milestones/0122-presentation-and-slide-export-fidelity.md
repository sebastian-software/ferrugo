# 0122: Presentation And Slide Export Fidelity

Status: done
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

Completed on 2026-06-25.

- Added four synthetic slide-export fixtures covering gradient title slides,
  layered image/tint/shadow slides, rotated chart callouts, and speaker notes.
- Added `fixtures/presentation-slide-manifest.tsv` for common slide-export
  subtype gates.
- Added native regression coverage for the new slide fixtures.
- Native slide gate renders 7/7 manifest rows without fallback or errors.
- Native benchmark has 0 budget failures with `--max-edge 160` and two
  iterations.
- PDFium visual oracle reports 2 exact matches and 5 fidelity blockers, with
  no native or PDFium render errors.
