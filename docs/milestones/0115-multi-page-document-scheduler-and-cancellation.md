# 0115: Multi-Page Document Scheduler And Cancellation

Status: todo
Phase: 20
Size: medium
Depends on: 0114

## Goal

Make multi-page native rendering ergonomic for consumers while keeping CPU,
memory, and cancellation behavior explicit.

## Scope

- Add a bounded scheduler for multiple requested page thumbnails.
- Share immutable document state across page jobs safely.
- Support cooperative cancellation and per-page error reporting.
- Add tests for long reports, mixed-success documents, and cancellation.

## Non-Goals

- Add unbounded parallel rendering.
- Require async runtimes in the core library API.
- Hide page-specific errors behind a single generic failure.

## Deliverables

- Multi-page scheduler API.
- Cancellation and partial-result documentation.
- Benchmark report for report-style PDFs.

## Acceptance Criteria

- Page jobs are bounded by configured concurrency and memory budgets.
- Cancellation stops new work and releases temporary buffers.
- Partial results preserve page-level status and error information.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run multi-page scheduler tests.
- Run long-document memory benchmarks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
