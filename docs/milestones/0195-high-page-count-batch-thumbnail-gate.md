# 0195: High Page Count Batch Thumbnail Gate

Status: todo
Phase: 36
Size: medium
Depends on: 0194

## Goal

Make high-page-count thumbnail generation predictable for local, server, and
viewer workflows using bounded memory and cancellable scheduling.

## Scope

- Add long-document thumbnail benchmarks with mixed page complexity.
- Verify cancellation, worker limits, and output ordering under load.
- Measure cache reuse across hundreds or thousands of pages.
- Document recommended batch-render settings.

## Non-Goals

- Optimize single-page visual fidelity in this milestone.
- Require every page in a malformed long document to render.
- Add unbounded prefetching.

## Deliverables

- High-page-count thumbnail benchmark report.
- Scheduler and cache tuning updates.
- Operational guidance for batch thumbnail consumers.

## Acceptance Criteria

- Batch thumbnail runs stay within configured memory budgets.
- Cancellation stops queued work without corrupting completed results.
- Throughput and failure reporting are stable across long documents.

## Validation

- Run native-only `cargo test`.
- Run batch thumbnail benchmark suite.
- Run cancellation and partial-result tests.
- Run memory profile for long documents.

## Completion Notes

Empty until done.
