# 0106: ICC Profile Cache And Transform Optimization

Status: done
Phase: 19
Size: medium
Depends on: 0105

## Goal

Make ICC-based color conversion fast, bounded, and reusable across pages and
documents.

## Scope

- Cache parsed ICC transforms by stable profile identity.
- Add limits for profile size, channel count, and transform workspace memory.
- Measure hot paths for image-heavy and office-export PDFs.
- Keep native output deterministic across supported platforms.

## Non-Goals

- Implement professional color-management UI.
- Cache untrusted profile data without validation.
- Optimize before correctness gates exist.

## Deliverables

- ICC cache and transform metrics.
- Memory-budget documentation update.
- Benchmark report for color-managed documents.

## Acceptance Criteria

- Repeated renders reuse validated ICC transforms.
- Cache eviction respects renderer memory budgets.
- Color-managed fixtures stay within configured render-time budgets.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run ICC color fixture comparisons.
- Run renderer benchmark suite for color-heavy PDFs.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-25.

- Added a bounded ICCBased image color-space path with profile byte limits,
  transform workspace limits, and cache entry limits.
- Added `IccTransformCache` metrics for hits, misses, evictions, and maximum
  validated transform workspace size.
- Added native memory diagnostics for ICC profile, transform workspace, and
  transform cache budgets.
- Added generated ICCBased RGB, Gray, and CMYK image fixtures plus native
  smoke coverage.
- Verified native benchmark budgets for the new ICC fixtures. RGB and Gray
  match PDFium exactly; ICCBased CMYK renders natively but remains a visual
  approximation gap.
- Report: `docs/reports/icc-cache-transform-2026-06-25.md`.
