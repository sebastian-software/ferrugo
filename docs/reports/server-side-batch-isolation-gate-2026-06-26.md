# Server Side Batch Isolation Gate 2026-06-26

Milestone: 0177

## Summary

Extended the native batch benchmark report with explicit isolation metadata for
server-side rendering. The report now states that batch jobs use per-job native
backend scope, `isolated-render` cache policy, no shared document state,
per-render timeout budget, and scheduled/skipped job counts for cooperative
cancellation.

This keeps the server path independent from WASM readiness and makes the
PDFium-free batch behavior machine-readable in CI artifacts.

## Implementation

Updated `benchmark-batch-native` with:

- `--cancel-after-jobs N` for deterministic scheduler cancellation smoke tests;
- `config.cancel_after_jobs` in JSON output;
- `isolation.cache_policy`;
- `isolation.scheduled_jobs`;
- `isolation.skipped_jobs`;
- `isolation.cancelled`;
- `isolation.backend_scope`;
- `isolation.shared_document_state`;
- `isolation.timeout_ms`.

The render hot path is unchanged. Each scheduled job still creates its own
`NativeBackend` and renders one input independently.

## Isolation Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/server-batch-manifest.tsv \
  --include-family small \
  --include-family mixed-size \
  --include-family image-heavy \
  --include-family repeated-resources \
  --include-family vector-stress \
  --repetitions 3 \
  --max-workers 4 \
  --max-in-flight-pixels 102400 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --fail-on-budget \
  --output target/server-batch-0177-isolation.json
```

Result:

| Metric | Value |
| --- | ---: |
| Inputs | 8 |
| Jobs | 24 |
| Native rendered | 24 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |
| Elapsed ms | 713.325 |
| Throughput jobs/sec | 33.645 |
| Mean ms | 47.365 |
| P50 ms | 26.514 |
| P95 ms | 190.848 |
| Max ms | 191.868 |
| Max output bytes | 78720 |

Isolation fields:

| Field | Value |
| --- | --- |
| Cache policy | `isolated-render` |
| Disk persistence | `false` |
| Scheduled jobs | `24` |
| Skipped jobs | `0` |
| Cancelled | `false` |
| Backend scope | `per-job` |
| Shared document state | `false` |
| Timeout ms | `5000` |

Family results:

| Family | Jobs | Native rendered | Mean ms | Max ms |
| --- | ---: | ---: | ---: | ---: |
| `small` | 6 | 6 | 12.511 | 24.422 |
| `mixed-size` | 9 | 9 | 33.268 | 44.620 |
| `image-heavy` | 3 | 3 | 45.346 | 50.439 |
| `repeated-resources` | 3 | 3 | 18.642 | 23.727 |
| `vector-stress` | 3 | 3 | 190.106 | 191.868 |

RSS fields were `null` in the sandboxed run, so the gate relies on hard
in-flight pixel bounds plus maximum output bytes.

## Cancellation Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated \
  --manifest fixtures/server-batch-manifest.tsv \
  --include-family small \
  --include-family mixed-size \
  --include-family image-heavy \
  --include-family repeated-resources \
  --include-family vector-stress \
  --repetitions 3 \
  --max-workers 4 \
  --max-in-flight-pixels 102400 \
  --max-edge 160 \
  --max-p95-ms 1000 \
  --max-errors 0 \
  --cancel-after-jobs 5 \
  --output target/server-batch-0177-cancelled.json
```

Result:

| Metric | Value |
| --- | ---: |
| Scheduled jobs | 5 |
| Skipped jobs | 19 |
| Native rendered | 5 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |
| P95 ms | 45.024 |
| Max output bytes | 78720 |

The cancellation gate confirms that the scheduler stops before additional work
is scheduled while already scheduled jobs remain isolated and successful.

## Validation

- `cargo test -p pdfrust-cli batch_benchmark -- --nocapture`
- Server batch isolation gate with `--fail-on-budget`.
- Server batch cancellation gate with `--cancel-after-jobs 5`.
- Native parallel cancellation tests are covered by the workspace test gate.
