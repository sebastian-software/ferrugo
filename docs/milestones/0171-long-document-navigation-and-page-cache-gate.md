# 0171: Long Document Navigation And Page Cache Gate

Status: todo
Phase: 32
Size: medium
Depends on: 0170

## Goal

Make long-document preview workflows efficient enough for typical viewer use
without retaining unbounded page state.

## Scope

- Add fixtures and synthetic documents with many pages, repeated resources, and
  mixed page complexity.
- Define page cache keys, eviction behavior, and memory accounting.
- Optimize first-page, next-page, and random-page render workflows.
- Add cancellation or early-exit behavior where render work can be abandoned.

## Non-Goals

- Build a full viewer UI.
- Keep every rendered page resident in memory.
- Add unsafe shared state for cache speed.

## Deliverables

- Long-document benchmark report.
- Page cache and eviction policy updates.
- Tests for repeated resource reuse and cache limits.

## Acceptance Criteria

- Long-document rendering has bounded peak memory.
- Repeated resources are reused without stale-page artifacts.
- First-page and random-page timings are measured and documented.

## Validation

- Run native-only `cargo test`.
- Run long-document benchmark suite.
- Run memory profile with cache pressure.
- Run visual comparison for sampled pages.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
