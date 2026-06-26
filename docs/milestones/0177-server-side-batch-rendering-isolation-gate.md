# 0177: Server-Side Batch Rendering Isolation Gate

Status: done
Phase: 33
Size: medium
Depends on: 0175

## Goal

Validate native renderer isolation for server-side batch thumbnail or preview
generation where many documents are rendered concurrently. This is a primary
PDFium-replacement path and does not depend on WASM readiness.

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
- Treat browser or WASM profile constraints as blockers for server-side batch
  correctness.

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

Completed on 2026-06-26.

Implemented explicit server batch isolation reporting for
`benchmark-batch-native`:

- per-job backend scope in the JSON report;
- `isolated-render` cache policy and disk-persistence status;
- no shared document state;
- scheduled and skipped job counts;
- cooperative scheduler cancellation via `--cancel-after-jobs`;
- timeout budget in the isolation block.

Added `docs/policies/server-batch-rendering.md` with recommended server gate
configuration, isolation rules, cancellation behavior, timeout handling, and
failure semantics.

Recorded the gate evidence in
`docs/reports/server-side-batch-isolation-gate-2026-06-26.md`:

- 24/24 jobs native rendered in the normal server batch gate;
- 0 fallbacks, 0 errors, 0 budget failures;
- cancellation gate scheduled 5 jobs and skipped 19 jobs without fallbacks or
  errors.

Validation completed:

- `cargo test -p pdfrust-cli batch_benchmark -- --nocapture`
- server batch isolation gate with `--fail-on-budget`
- server batch cancellation gate with `--cancel-after-jobs 5`
