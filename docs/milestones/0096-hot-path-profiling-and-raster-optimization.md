# 0096: Hot Path Profiling And Raster Optimization

Status: todo
Phase: 16
Size: medium
Depends on: 0095

## Goal

Optimize measured renderer hot paths while preserving safe Rust boundaries and
predictable memory behavior.

## Scope

- Profile path rasterization, image scaling, blending, glyph rendering, and
  stream decoding on representative fixtures.
- Remove redundant allocation, cloning, and intermediate buffers.
- Add focused benchmarks for optimized paths.
- Document any unsafe-free or unsafe-minimized optimization decisions.

## Non-Goals

- Rewrite stable code without profiler evidence.
- Add unsafe code without a safety review and measurable benefit.
- Tune only microbenchmarks that do not affect document rendering.

## Deliverables

- Profiling report.
- Targeted optimization patches.
- Benchmark deltas before and after changes.

## Acceptance Criteria

- Each optimization has measured impact on a document category.
- No optimization weakens renderer error handling or bounds.
- Clippy and tests remain clean under all targets and features.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run renderer benchmarks before and after changes.
- Run profiling capture for at least one representative fixture.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
