# 0183: Mixed Vector Raster Transparency Edge Cases

Status: todo
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

## Completion Notes

Empty until done.
