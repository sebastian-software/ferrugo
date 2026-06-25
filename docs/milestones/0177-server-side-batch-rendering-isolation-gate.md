# 0177: Server-Side Batch Rendering Isolation Gate

Status: todo
Phase: 33
Size: medium
Depends on: 0176

## Goal

Validate native renderer isolation for server-side batch thumbnail or preview
generation where many documents are rendered concurrently.

## Scope

- Add batch rendering stress tests for many small documents and fewer large
  documents.
- Define concurrency, memory, cancellation, timeout, and per-document isolation
  budgets.
- Audit shared caches for thread safety, memory retention, and cross-document
  contamination.
- Document recommended server-side configuration and failure handling.

## Non-Goals

- Add a hosted rendering service.
- Use process-global mutable state without synchronization.
- Optimize raw throughput ahead of isolation and bounded resource use.

## Deliverables

- Server-side isolation benchmark report.
- Batch renderer configuration guidance.
- Concurrency, timeout, and cache safety tests.

## Acceptance Criteria

- Batch rendering keeps per-document failures isolated.
- Timeouts and cancellations leave no retained corrupted state.
- Shared caches are bounded and thread-safe.

## Validation

- Run native-only `cargo test`.
- Run batch stress suite.
- Run concurrency and cancellation tests.
- Run memory profile under batch load.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
