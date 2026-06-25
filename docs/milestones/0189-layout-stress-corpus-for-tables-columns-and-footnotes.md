# 0189: Layout Stress Corpus For Tables Columns And Footnotes

Status: todo
Phase: 35
Size: medium
Depends on: 0188

## Goal

Expand typical-document coverage for dense layouts with tables, columns,
footnotes, headers, and repeated page furniture.

## Scope

- Add generated and public fixtures for reports, statements, academic pages, and
  dense business documents.
- Classify failures by text positioning, clipping, line art, font, and image
  interactions.
- Tune visual thresholds for dense but low-risk text drift.
- Identify targeted renderer fixes for high-frequency layout gaps.

## Non-Goals

- Build a semantic table extractor.
- Require exact subpixel parity for every glyph.
- Accept unreadable text because the page structure is complex.

## Deliverables

- Dense layout corpus report.
- Fixture metadata additions.
- Ranked renderer gap list.

## Acceptance Criteria

- Dense report-style documents are represented in the corpus.
- Regressions in table lines, columns, and footnote regions are visible.
- Support matrix distinguishes visual fidelity from semantic extraction gaps.

## Validation

- Run native-only `cargo test`.
- Run dense-layout visual comparisons.
- Run text placement diagnostics where available.
- Review visual thresholds for over-broad acceptance.

## Completion Notes

Empty until done.
