# 0126: Scientific Paper And Long Report Layout Coverage

Status: todo
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

Empty until done.
