# Low-End Device Reliability Sweep

Date: 2026-06-29
Milestone: 0217

## Summary

Milestone 0217 validates low-end and constrained Rust-native rendering as a
secondary reliability profile. The server-side PDFium replacement path remains
primary; WASM and reduced-device findings are compatibility signals unless they
surface shared renderer correctness, safety, or unbounded-resource defects.

New artifact:

- `fixtures/low-end-reliability-profile-matrix.tsv`
- `scripts/check_low_end_reliability_matrix.sh`

## Profile Matrix

| Profile | Workflow | Artifact | Result | Blocking scope |
| --- | --- | --- | --- | --- |
| `low-memory-summary` | Cross-producer typical workflows | `target/low-end-0217-low-memory-summary.json` | passed | server-constrained |
| `low-memory-repeat` | Repeated cross-producer renders | `target/low-end-0217-low-memory-repeat.json` | passed | server-constrained |
| `server-constrained-batch` | High-page-count page fanout | `target/low-end-0217-server-constrained-batch.json` | passed | server-primary |
| `wasm-smoke` | Browser thumbnail smoke | `target/wasm-0132-smoke.json` | passed | secondary-profile |
| `deterministic-reduced-canvas` | Repeated reduced-canvas render | `target/low-end-0217-deterministic-a.png` | passed | server-constrained |

## Low-Memory Typical Workflow Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --fail-on-fallback --max-edge 120 --native-profile low-memory --output target/low-end-0217-low-memory-summary.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 20 | 20 | 0 | 0 |

All five supported cross-producer workflow families passed with a 1.000 native
pass rate.

## Low-Memory Repeat Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --native-profile low-memory --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/low-end-0217-low-memory-repeat.json
```

Result:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 20 | 20 | 0 | 0 | 0 |

Family first-render means ranged from 5.363 ms to 13.677 ms. Repeat means
ranged from 5.282 ms to 13.720 ms. The report kept the `isolated-render` cache
policy and low-memory profile in every cache key.

## Server-Constrained Batch Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 2 --pages-per-input 12 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/low-end-0217-server-constrained-batch.json
```

Result:

| Total inputs | Total jobs | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5 | 50 | 50 | 0 | 0 | 0 |

The constrained batch used per-job backends, no shared document state,
`max_workers = 2`, and `max_in_flight_pixels = 51200`. P95 latency was
4.861 ms, below the 1000 ms gate.

## WASM Secondary Profile

Command:

```sh
bash scripts/check_wasm_smoke.sh
```

Result:

| Metric | Measured | Gate |
| --- | ---: | ---: |
| Artifact size bytes | 730359 | 4194304 |
| WebAssembly compile ms | 1.481 | 250 |
| WebAssembly instantiate ms | 0.056 | 100 |
| Smoke render ms | 5.980 | 250 |
| Smoke output | 96x51 | 96 max edge |

## Deterministic Reduced Canvas

Commands:

```sh
cargo run -p pdfrust-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-a.png
cargo run -p pdfrust-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-b.png
cmp -s target/low-end-0217-deterministic-a.png target/low-end-0217-deterministic-b.png
```

The two reduced-canvas PNG outputs were byte-identical. The single-render CLI
does not expose `--native-profile`; profile-specific repeated-render reliability
is covered by `benchmark-repeat-native --native-profile low-memory`.

## Degradation Policy

- Supported low-end server profiles must return native output or typed errors;
  they must not panic or retry through PDFium.
- Budget exhaustion remains visible through `renderer.memory-budget` typed
  unsupported outcomes.
- WASM findings remain secondary unless they expose shared renderer safety,
  correctness, or unbounded-resource behavior.
- RSS fields can remain unavailable on macOS batch runs; explicit pixel/output
  budgets are still enforced by the benchmark gate.

## Validation

Commands run:

```sh
bash scripts/check_low_end_reliability_matrix.sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --fail-on-fallback --max-edge 120 --native-profile low-memory --output target/low-end-0217-low-memory-summary.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-repeat-native fixtures/generated --manifest fixtures/cross-producer-fusion-manifest.tsv --include-family fused-report --include-family fused-table-statement --include-family fused-form --include-family fused-scan --include-family fused-dashboard-map --native-profile low-memory --repetitions 2 --max-edge 120 --max-first-ms 1000 --max-repeat-mean-ms 1000 --max-errors 0 --fail-on-budget --output target/low-end-0217-low-memory-repeat.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/high-page-count-batch-manifest.tsv --include-family long-document --include-family book --include-family email-thread --include-family repeated-resources --include-family report-statement --repetitions 2 --pages-per-input 12 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 120 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --native-profile low-memory --output target/low-end-0217-server-constrained-batch.json
cargo run -p pdfrust-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-a.png
cargo run -p pdfrust-cli --no-default-features -- render-native fixtures/generated/business-invoice-dense.pdf --max-edge 96 --output target/low-end-0217-deterministic-b.png
cmp -s target/low-end-0217-deterministic-a.png target/low-end-0217-deterministic-b.png
bash scripts/check_wasm_smoke.sh
cargo fmt --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check -- fixtures/low-end-reliability-profile-matrix.tsv scripts/check_low_end_reliability_matrix.sh docs/policies/renderer-memory-budgets.md docs/policies/server-batch-rendering.md docs/milestones/README.md docs/milestones/0217-low-end-device-reliability-sweep.md docs/reports/low-end-device-reliability-sweep-2026-06-29.md
```
