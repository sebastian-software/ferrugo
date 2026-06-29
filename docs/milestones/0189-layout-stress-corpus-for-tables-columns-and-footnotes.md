# 0189: Layout Stress Corpus For Tables Columns And Footnotes

Status: done
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

Completed on 2026-06-29.

- Added `fixtures/generated/layout-columns-footnotes-table-stress.pdf`, a
  deterministic dense report page with two text columns, header/footer
  furniture, a figure interrupt, ruled table geometry, and footnote-region
  small text.
- Added `fixtures/layout-stress-manifest.tsv` so dense tables, spreadsheet
  grids, two-column pages, footnotes, and page furniture can be gated together.
- Added focused native render coverage for the new stress page and classified
  it in the main corpus manifest as `office-export` with
  `expected:native`.
- Recorded support and fidelity evidence in
  `docs/reports/layout-stress-corpus-2026-06-29.md`.

Native rendering is supported for the focused layout-stress set. The Poppler
visual oracle still reports dense-layout fidelity blockers for table text,
two-column text placement, and footnote-region drift, so the support matrix
separates renderability from visual parity and from out-of-scope semantic table
or reading-order extraction.
