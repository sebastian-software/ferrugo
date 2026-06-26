# 0159: SIMD Raster Hot Path Evaluation

Status: done
Phase: 29
Size: medium
Depends on: 0158

## Goal

Evaluate SIMD and platform-accelerated implementations for raster hot paths
while preserving deterministic, portable Rust-native behavior.

## Scope

- Profile blend, fill, image conversion, alpha, and color transform hot paths.
- Prototype portable SIMD or feature-gated platform acceleration where useful.
- Preserve scalar fallbacks and deterministic test expectations.
- Compare speedups against code complexity and maintenance cost.

## Non-Goals

- Require nightly Rust for normal builds.
- Remove scalar fallback paths.
- Optimize unmeasured paths.

## Deliverables

- SIMD evaluation report.
- Accepted acceleration patches or explicit rejection notes.
- Benchmark results across representative document families.

## Acceptance Criteria

- Any SIMD path has a scalar fallback and identical output within documented
  tolerances.
- Benchmarks show meaningful improvement before code is retained.
- Platform feature detection is explicit and testable.

## Validation

- Run scalar and accelerated test modes.
- Run renderer benchmark suite.
- Run visual comparison subset.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

- Evaluated scalar blend, source-over, image sample conversion, transparency,
  vector-stress, scan, and repeated-resource hot paths.
- Rejected a retained SIMD/platform-accelerated path for this milestone because
  normal builds must honor the Rust 1.81 MSRV and the measured evidence does
  not justify dual scalar/accelerated maintenance yet.
- Kept scalar rendering as the only production path.
- Added `docs/reports/simd-raster-hot-path-evaluation-2026-06-26.md` with
  benchmark, visual-subset, and rejection evidence.
