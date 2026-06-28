# 0183: Mixed Vector Raster Transparency Edge Cases

Status: in-progress
Phase: 34
Size: medium
Depends on: 0182

## Goal

Close fidelity gaps in common pages that combine vector artwork, raster images,
soft masks, clipping, and transparency groups.

## Scope

- Add mixed vector/raster transparency fixtures from office, browser, and design
  tool producers.
- Audit compositing paths for intermediate allocation size and reuse.
- Improve or explicitly type unsupported edge cases around nested masks and
  clipped images.
- Update visual thresholds for affected document families.

## Non-Goals

- Implement every blend or prepress feature in one milestone.
- Optimize unrelated raster paths.
- Hide transparency failures behind broad accepted drift.

## Deliverables

- Mixed transparency corpus report.
- Renderer fixes or typed unsupported gaps.
- Memory notes for intermediate surfaces.

## Acceptance Criteria

- Common mixed vector/raster pages pass documented visual gates.
- Intermediate surface allocation stays within renderer budgets.
- Remaining gaps are specific and actionable.

## Validation

- Run native-only `cargo test`.
- Run transparency fixture visual comparisons.
- Run benchmark and memory profiles for affected pages.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Progress Notes

Native baseline slice started on 2026-06-28.

- Added `fixtures/mixed-vector-raster-transparency-manifest.tsv` with 8
  existing generated fixtures covering browser raster/vector output, high-DPI
  previews, rotated soft-mask images, map overlays, office clipped transparency
  groups, repeated office vector effects, slide image shadows, and image soft
  masks.
- Added
  `docs/reports/mixed-vector-raster-transparency-2026-06-28.md`.
- Native gate: 8 total, 8 native rendered, 0 fallback required, 0 errors.
- Benchmark gate: 8 total, 8 native rendered, 0 fallback required, 0 errors,
  0 budget failures under `--max-edge 160`, two iterations, `--max-ms 1000`,
  and `--max-output-bytes 1048576`.
- Poppler visual baseline: 8 total, 0 exact, 2 accepted drift, 6 blockers,
  0 native errors, 0 reference errors, 0 both errors.
- Next fidelity focus: reduce `office-vector-clipped-transparency-group.pdf`
  and `map-transparent-zoning-overlay.pdf` blockers before broadening the
  fixture slice.

## Completion Notes

Empty until done.
