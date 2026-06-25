# 0126: Scientific Paper And Long Report Layout Coverage

Status: done
Phase: 23
Size: medium
Depends on: 0125

## Goal

Validate native rendering for research papers, whitepapers, and long reports
with multi-column text, equations, figures, footnotes, and references.

## Scope

- Add fixtures for multi-column papers and long report layouts.
- Cover embedded subset fonts, symbols, equations as vector/text, and figures.
- Verify first-page and representative interior-page rendering.
- Track failures separately for font, layout, image, and vector features.

## Non-Goals

- Extract citations or document structure.
- Interpret equation semantics.
- Render every TeX package edge case.

## Deliverables

- Scientific and long-report fixture family.
- Multi-page sampling report.
- Font and symbol blocker backlog.

## Acceptance Criteria

- Typical papers and reports produce useful native thumbnails.
- Multi-column text, figures, and symbols retain recognizable placement.
- Long-report sampling keeps memory and page scheduling bounded.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run paper/report corpus comparisons.
- Run multi-page sampling benchmark.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated fixtures for two-column papers, equation/figure pages,
  reference and footnote layouts, and a three-page long-report sampling case.
- Added `fixtures/scientific-report-manifest.tsv` with eight focused rows
  across `paper`, `equation-figure`, `long-report`, and
  `references-footnotes` families.
- Added native regression coverage for first-page scientific/report layouts and
  parallel sampling of pages 0 and 2 in the long-report fixture.
- Native fallback gate: 8/8 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 8/8 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle: 0 exact matches, 1 accepted drift, 7 strict-threshold
  blockers, 0 native render errors, 0 PDFium render errors.
- Report: `docs/reports/scientific-report-fidelity-2026-06-25.md`.
