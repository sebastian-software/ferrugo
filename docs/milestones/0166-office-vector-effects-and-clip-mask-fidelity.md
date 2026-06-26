# 0166: Office Vector Effects And Clip Mask Fidelity

Status: done
Phase: 31
Size: medium
Depends on: 0165

## Goal

Improve native rendering for common office-export vector effects that appear in
slides, reports, charts, and diagrams.

## Scope

- Add fixtures for grouped vector shapes, nested clipping, soft edges,
  transparency, and repeated decorative effects.
- Audit existing display-list and raster paths for clipping stack correctness.
- Add targeted fixes for office-vector cases that can be implemented without
  broad renderer rewrites.
- Document unsupported vector effects with typed reasons.

## Non-Goals

- Build a full presentation editor model.
- Implement arbitrary proprietary office semantics beyond the exported PDF.
- Relax memory budgets for complex vector pages.

## Deliverables

- Office vector effect fixtures.
- Native renderer fixes for bounded clip-mask cases.
- Visual fidelity report for the office-vector subset.

## Acceptance Criteria

- Representative office vector fixtures render without runtime fallback.
- Clip masks and transparency preserve bounded memory usage.
- Remaining failures are typed and documented.

## Validation

- Run native-only `cargo test`.
- Run office-export fixture visual comparison.
- Run vector stress subset benchmarks.
- Run memory budget checks for nested clipping.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Added four generated office-vector fixtures for grouped shapes, nested
  clipping, clipped transparency groups, and repeated decorative effects.
- Added `fixtures/office-vector-effects-manifest.tsv` and included the new
  fixtures in the main corpus manifest under `office-export`.
- Fixed transparency-group compositing so parent clipping paths are applied with
  bounded supersampled coverage and no separate mask allocation.
- Added native regression tests for the office-vector fixture set and clipped
  transparency-group behavior.
- Added
  `docs/reports/office-vector-effects-clip-mask-fidelity-2026-06-26.md`.
