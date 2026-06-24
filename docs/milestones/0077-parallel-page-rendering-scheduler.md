# 0077: Parallel Page Rendering Scheduler

Status: done
Phase: 12
Size: medium
Depends on: 0076

## Goal

Add bounded parallel rendering for multi-page workloads without compromising
memory budgets or deterministic errors.

## Scope

- Define thread-safe shared document data and per-page render state.
- Add a configurable worker count and memory-aware scheduling policy.
- Keep caches synchronized without global unbounded locks.
- Add tests for deterministic output and cancellation behavior.

## Non-Goals

- Parallelize every operation within a single page.
- Add async runtime requirements to the core renderer.
- Ignore memory ceilings for throughput.

## Deliverables

- Parallel page rendering scheduler.
- Benchmarks for sequential versus parallel multi-page rendering.
- Documentation for worker and memory configuration.

## Acceptance Criteria

- Multi-page rendering can use multiple workers under explicit limits.
- Results are stable compared with sequential rendering.
- Scheduler backs off or fails predictably when memory budgets are tight.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run multi-page performance benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: add bounded parallel page scheduler` implementation,
the `test: compare parallel scheduler output` stability test, and the
`docs: complete parallel scheduler coverage` report update.

- Added `ParallelRenderOptions`, `ParallelRenderResult`, and
  `render_pages_parallel` to the native backend.
- Shared input bytes across scoped page workers while preserving per-page render
  state and requested result order.
- Added memory-budget backoff and deterministic memory-budget failure through
  the existing `renderer.memory-budget` unsupported bucket.
- Added tests for ordered results, byte-for-byte parity with sequential renders,
  worker backoff, too-small budgets, and deterministic page errors.
- Recorded validation and timing evidence in
  `docs/reports/parallel-page-scheduler-coverage-2026-06-24.md`.
