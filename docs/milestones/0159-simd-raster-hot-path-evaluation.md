# 0159: SIMD Raster Hot Path Evaluation

Status: todo
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

Empty until done.
