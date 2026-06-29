# 0195: High Page Count Batch Thumbnail Gate

Status: done
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

- Added `benchmark-batch-native --pages-per-input N` to fan out ordered page
  jobs per input document while preserving the default one-page behavior.
- Batch job ordering is deterministic by repetition, input path, and page
  index; manifest page counts bound fanout for committed fixtures.
- Batch JSON now records `pages_per_input` in the config block.
- Added focused manifest `fixtures/high-page-count-batch-manifest.tsv`.
- Updated batch rendering policy and native backend docs with high-page-count
  gate guidance.
- Report:
  `docs/reports/high-page-count-batch-thumbnail-gate-2026-06-29.md`.
- Validation:
  - `cargo test -p ferrugo-cli batch -- --nocapture`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 10 --pages-per-input 12 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/high-page-count-0195-batch.json`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 10 --pages-per-input 12 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --cancel-after-jobs 25 --output target/high-page-count-0195-cancelled.json`
  - `cargo run -p ferrugo-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 3 --pages-per-input 12 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/high-page-count-0195-low-memory.json`
