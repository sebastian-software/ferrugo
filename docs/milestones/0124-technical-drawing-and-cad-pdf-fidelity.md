# 0124: Technical Drawing And CAD PDF Fidelity

Status: done
Phase: 22
Size: medium
Depends on: 0123

## Goal

Cover technical drawings and CAD-style PDFs that stress thin vector geometry,
large coordinate systems, clipping, and repeated symbols.

## Scope

- Add technical drawing fixtures with fine lines, hatches, labels, and symbols.
- Validate large page boxes, user units, and precision-sensitive transforms.
- Track path flattening, clipping, dash, and join fidelity.
- Keep vector workloads bounded by explicit segment and raster budgets.

## Non-Goals

- Parse CAD source formats.
- Support interactive layer toggling beyond existing optional-content policy.
- Guarantee print-production exactness for every engineering drawing.

## Deliverables

- Technical drawing fixture family.
- Vector fidelity report for drawing-style pages.
- Budget notes for large coordinate systems and repeated geometry.

## Acceptance Criteria

- Typical technical drawing thumbnails render without geometry collapse.
- Fine strokes remain visible where PDFium renders visible marks.
- Excessive geometry fails with typed budget errors instead of unbounded work.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run technical drawing visual comparisons.
- Run vector and memory budget benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added four generated technical drawing fixtures for dimensioned linework,
  clipped hatch sections, large-coordinate plans, and repeated symbols.
- Added `fixtures/technical-drawing-manifest.tsv` with eight focused rows,
  combining the new fixtures with existing dashed-stroke, clipped-path,
  vector-stress, and UserUnit baselines.
- Added a native regression test that asserts expected dimensions and visible
  non-background pixels for the new drawing fixtures.
- Native fallback gate: 8/8 rendered natively, 0 fallbacks, 0 errors.
- Native benchmark gate: 8/8 rendered natively, 0 budget failures at
  `--max-edge 160`, two iterations, `--max-ms 1000`, and
  `--max-output-bytes 1048576`.
- PDFium visual oracle: 2 exact matches, 6 strict-threshold blockers, 0 native
  render errors, 0 PDFium render errors.
- Report: `docs/reports/technical-drawing-fidelity-2026-06-25.md`.
