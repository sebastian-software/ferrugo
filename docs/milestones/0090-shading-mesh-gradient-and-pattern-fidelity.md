# 0090: Shading Mesh Gradient And Pattern Fidelity

Status: completed
Phase: 15
Size: medium
Depends on: 0089

## Goal

Improve native rendering for gradients, shadings, and reusable patterns found
in charts, presentations, and branded documents.

## Scope

- Extend axial and radial shading coverage where already partially supported.
- Triage mesh shading usage and choose approximation or unsupported behavior.
- Improve tiling pattern placement, transforms, and cache policy.
- Add fixtures from chart, presentation, and letterhead-style documents.

## Non-Goals

- Implement every shading type without corpus evidence.
- Add unbounded pattern raster caches.
- Sacrifice deterministic rendering for approximate shortcuts.

## Deliverables

- Shading and pattern fidelity improvements.
- Pattern cache budget notes.
- Visual comparison report.

## Acceptance Criteria

- Common gradients and tiling patterns match PDFium within documented drift.
- Unsupported mesh cases fail explicitly or have documented approximation.
- Pattern rendering remains bounded under repeated fills.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run pattern and shading visual comparisons.
- Run pattern-cache memory checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Completed 2026-06-25.
- Added a generated unsupported mesh-shading fixture and verified it reports
  `graphics.pattern-shading` instead of silently rendering incorrect output.
- Tightened visual-diff classification for full-field low-amplitude gradient
  drift, keeping high-delta differences as blockers.
- Verified the existing axial/radial gradient and tiling-pattern fixtures
  against PDFium at milestone scale; axial and radial gradients are accepted
  drift, while the tiling pattern is exact.
- Recorded the evidence in
  `docs/reports/shading-pattern-fidelity-2026-06-25.md`.
