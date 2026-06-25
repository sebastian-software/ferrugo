# Multi-Page Scheduler Cancellation Report

Date: 2026-06-25.
Milestone: 0115.

## Summary

Milestone 0115 makes the existing native multi-page scheduler more ergonomic
for consumers that need partial page results and explicit cancellation behavior.
The strict `render_pages_parallel` API remains available for callers that want
all-success semantics. The new `render_pages_parallel_partial` API preserves
page-level success and error outcomes in request order.

Cancellation is cooperative and checked before each worker batch is scheduled.
Already-started page jobs finish normally, which keeps cleanup deterministic
and avoids introducing async runtime requirements into the core renderer.

## Implementation

- Added `RenderCancellation`.
- Added `ParallelPageResult`.
- Added `ParallelRenderPartialResult`.
- Added `render_pages_parallel_partial`.
- Rebuilt `render_pages_parallel` on top of the partial scheduler while
  preserving its existing strict error behavior.
- Added scheduler tests for mixed page success/error results and pre-scheduling
  cancellation.

The scheduler remains bounded by `ParallelRenderOptions::max_workers` and
`max_in_flight_pixels`. If the memory budget cannot schedule even one page, the
same stable `renderer.memory-budget` unsupported bucket is returned.

## Evidence

Benchmark artifact: `target/multipage-0115-benchmark.json`

- Total: 103 fixtures.
- Native rendered: 96.
- Fallback required: 6.
- Errors: 1 encrypted fixture.
- Budget failures: 7 existing fallback/error cases.

Supported-family gate artifact: `target/multipage-0115-supported-gate.json`

- Total: 45.
- Native rendered: 45.
- Fallback required: 0.
- Families: `browser-print`, `office-export`, `form`.

Focused behavior covered by tests:

- Requested page order remains stable.
- Parallel results match sequential page renders.
- Worker count backs off under tight pixel budgets.
- A budget too small for one page returns `renderer.memory-budget`.
- Strict mode reports the earliest requested page error.
- Partial mode preserves a successful page and a later malformed page error.
- Pre-scheduling cancellation returns an empty cancelled partial result.

## Validation Commands

- `cargo fmt`
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo check --workspace --no-default-features`
- `cargo test -p pdfrust-native parallel -- --nocapture`
- `cargo test -p pdfrust-native`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/multipage-0115-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/multipage-0115-supported-gate.json`

## Follow-Ups

- Add a consumer-facing CLI command for multi-page batches if product usage
  needs it.
- Add deeper in-render cancellation checkpoints only after profiling shows page
  jobs can run long enough to justify the extra checks.
- Keep the core scheduler runtime-free; async adapters can wrap the synchronous
  API in downstream integration crates.
