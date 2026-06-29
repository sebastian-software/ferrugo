# Server And WASM Scheduler Tuning Gate

Date: 2026-06-29
Milestone: 0218

## Summary

Milestone 0218 validates scheduler tuning for the server-side Rust-native
renderer first. WASM remains a secondary compatibility profile unless a failure
exposes shared scheduler correctness, cancellation, or bounded-resource defects.

New artifacts:

- `fixtures/scheduler-tuning-profile-matrix.tsv`
- `scripts/check_scheduler_tuning_matrix.sh`

## Profile Matrix

| Profile | Workflow | Artifact | Result | Blocking scope |
| --- | --- | --- | --- | --- |
| `server-batch` | Server throughput | `target/scheduler-0218-server-batch.json` | passed | server-primary |
| `cancellation` | High-page-count cancellation | `target/scheduler-0218-cancellation.json` | passed | server-primary |
| `low-memory-batch` | Server-constrained batch | `target/scheduler-0218-low-memory-batch.json` | passed | server-constrained |
| `repeat-cache` | Repeated typical workflows | `target/scheduler-0218-repeat-cache.json` | passed | server-constrained |
| `wasm-smoke` | Browser thumbnail smoke | `target/wasm-0132-smoke.json` | passed | secondary-profile |

## Server Batch Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 3 --pages-per-input 1 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-server-batch.json
```

Result:

| Total inputs | Total jobs | Native rendered | Fallbacks | Errors | Budget failures | P95 ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 24 | 24 | 0 | 0 | 0 | 137.507 |

The gate used per-job native backends, `isolated-render` cache policy, and
`max_in_flight_pixels = 102400`. Effective scheduling stayed within the worker
and pixel budgets without using shared document state.

## Cancellation Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 3 --pages-per-input 12 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --cancel-after-jobs 20 --output target/scheduler-0218-cancellation.json
```

Result:

| Scheduled jobs | Skipped jobs | Cancelled | Native rendered | Errors | Shared document state | P95 ms |
| ---: | ---: | --- | ---: | ---: | --- | ---: |
| 20 | 55 | true | 20 | 0 | false | 5.328 |

Cancellation remained cooperative: already scheduled page jobs finished, no
new jobs were scheduled after the boundary, and the report retained the
per-job backend and isolated cache policy.

## Low-Memory Batch Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 2 --pages-per-input 1 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/scheduler-0218-low-memory-batch.json
```

Result:

| Total inputs | Total jobs | Native rendered | Fallbacks | Errors | Budget failures | P95 ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 16 | 16 | 0 | 0 | 0 | 55.308 |

The constrained server profile stayed within `max_workers = 2` and
`max_in_flight_pixels = 51200`. This remains a reliability guard rather than
the default throughput target.

## Repeat Cache Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-repeat-cache.json
```

Result:

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 20 | 20 | 0 | 0 | 0 |

Family first-render means ranged from 5.403 ms to 13.977 ms. Repeat means
ranged from 5.369 ms to 13.795 ms. Cache keys include document identity, page
index, max edge, background, renderer version, and native profile.

## WASM Secondary Profile

Command:

```sh
bash scripts/check_wasm_smoke.sh
```

Result:

| Metric | Measured | Gate |
| --- | ---: | ---: |
| Artifact size bytes | 730359 | 4194304 |
| WebAssembly compile ms | 1.590 | 250 |
| WebAssembly instantiate ms | 0.061 | 100 |
| Smoke render ms | 6.277 | 250 |
| Smoke output | 96x51 | 96 max edge |

The WASM smoke stayed within the packaging and smoke-render budget. It remains
a secondary profile check for this milestone.

## Regression Coverage

The CLI batch benchmark now has a focused regression test for unschedulable
pixel budgets. A batch configuration whose `max_in_flight_pixels` cannot fit a
single render job returns the typed budget failure
`benchmark budget failure: batch memory budget cannot schedule one render job`
instead of silently overcommitting memory.

## Scheduler Defaults

- Server throughput gate: `--max-workers 4`, `--max-in-flight-pixels 102400`,
  `--max-edge 160`, `--pages-per-input 1`.
- High-page-count cancellation gate: same server pixel budget, `--max-edge 120`,
  `--pages-per-input 12`, explicit `--cancel-after-jobs`.
- Server-constrained gate: `--max-workers 2`, `--max-in-flight-pixels 51200`,
  `--max-edge 120`, `--native-profile low-memory`.
- Repeat cache gate: isolated render cache policy, deterministic cache keys,
  no disk persistence.
- WASM gate: package and smoke-render budget only; no server release blocking
  unless shared scheduler correctness or bounded-resource behavior regresses.

## Validation

Commands run:

```sh
bash scripts/check_scheduler_tuning_matrix.sh
cargo test -p pdfrust-cli batch_benchmark -- --nocapture
cargo test -p pdfrust-cli repeat_benchmark -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 3 --pages-per-input 1 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-server-batch.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 3 --pages-per-input 12 --max-workers 4 --max-in-flight-pixels 102400 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --cancel-after-jobs 20 --output target/scheduler-0218-cancellation.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 2 --pages-per-input 1 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/scheduler-0218-low-memory-batch.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/scheduler-0218-repeat-cache.json
bash scripts/check_wasm_smoke.sh
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check -- crates/pdfrust-cli/src/main.rs fixtures/scheduler-tuning-profile-matrix.tsv scripts/check_scheduler_tuning_matrix.sh docs/policies/server-batch-rendering.md docs/backend/native.md docs/milestones/README.md docs/milestones/0218-server-and-wasm-scheduler-tuning-gate.md docs/reports/server-wasm-scheduler-tuning-2026-06-29.md
```
