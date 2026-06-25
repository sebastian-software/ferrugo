# 0096: Hot Path Profiling And Raster Optimization

Status: done
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

Completed on 2026-06-25.

- Added unsafe-free device-pixel bounds for native fill, tiling-pattern fill,
  and stroke raster loops so small vector items no longer scan the full raster
  surface.
- Added focused helper tests for clipped, padded, and off-raster pixel bounds.
- Captured before/after benchmark artifacts for
  `fixtures/generated/vector-stress.pdf`:
  `target/hotpath-0096-vector-stress-before.json` and
  `target/hotpath-0096-vector-stress-after.json`.
- Improved the representative vector-stress benchmark from 2833.258 ms to
  184.189 ms mean render time with the same 76800 output bytes and no budget
  violations.
- Captured a profiling sample at
  `target/hotpath-0096-vector-stress.sample.txt`; remaining hot frames are the
  expected geometry tests inside fill and stroke coverage checks.
- See `docs/reports/hot-path-profiling-raster-optimization-2026-06-25.md`.
