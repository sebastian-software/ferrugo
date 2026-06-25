# Server Batch Throughput 2026-06-25

Milestone: 0133.

## Decision

The CLI now has a native server-batch benchmark for many independent PDF
inputs. `benchmark-batch-native` uses explicit worker and in-flight pixel
budgets, repeats a focused fixture set, reports per-input typed outcomes, and
records throughput, latency distribution, output high-water, and RSS samples
when process inspection is available.

The benchmark does not share untrusted document state across inputs. Each job
creates its own native backend and renders one requested page. This keeps the
server-side default isolated while still allowing worker-bounded throughput
measurement.

## Batch Corpus

Artifact: `fixtures/server-batch-manifest.tsv`

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `small` | 2 | High-count small text/table baseline |
| `mixed-size` | 3 | Business, scientific, and chart exports |
| `image-heavy` | 1 | Mobile scan decoded-image cost |
| `repeated-resources` | 1 | Repeated resource/cache observation |
| `vector-stress` | 1 | Latency tail observation |

## Gate Evidence

Artifact: `target/server-batch-0133-benchmark.json`

Configuration:

| Setting | Value |
| --- | ---: |
| Repetitions | 2 |
| Workers | 2 |
| Max in-flight pixels | 51200 |
| Max edge | 160 |
| Max p95 ms | 1000 |
| Max errors | 0 |

Summary:

| Metric | Value |
| --- | ---: |
| Total inputs | 8 |
| Total jobs | 16 |
| Native rendered | 16 |
| Fallback required | 0 |
| Errors | 0 |
| Budget failures | 0 |
| Elapsed ms | 605.888 |
| Throughput jobs/sec | 26.408 |

Latency:

| Metric | Value |
| --- | ---: |
| Mean ms | 45.157 |
| P50 ms | 26.687 |
| P95 ms | 184.736 |
| Max ms | 184.736 |

Memory:

| Metric | Value |
| --- | ---: |
| RSS start KiB | 2848 |
| RSS high-water KiB | 5664 |
| RSS end KiB | 5664 |
| Max in-flight pixels | 51200 |
| Max output bytes | 78720 |

RSS sampling uses `ps` and may return `null` in restricted sandboxes. The gate
still records the hard scheduling bound and maximum output bytes in those
contexts. The measurement above was captured in an unsandboxed run.

## Family Results

| Family | Jobs | Native rendered | Mean ms | Max ms | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `small` | 4 | 4 | 12.125 | 23.805 | 0 |
| `mixed-size` | 6 | 6 | 31.641 | 39.216 | 0 |
| `image-heavy` | 2 | 2 | 41.471 | 41.551 | 0 |
| `repeated-resources` | 2 | 2 | 15.965 | 15.983 | 0 |
| `vector-stress` | 2 | 2 | 184.644 | 184.736 | 0 |

## Recommendations

- Use two workers for CI/server smoke gates at `max_edge 160` until broader
  throughput profiling justifies higher defaults.
- Keep `max_in_flight_pixels` tied to worker count and thumbnail size rather
  than host CPU count alone.
- Treat vector-stress inputs as latency-tail drivers. They are useful in the
  batch gate but should not dominate small-document throughput expectations.
- Keep cache sharing out of the server-side default until an explicit
  tenant-safe cache policy lands.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-cli/src/main.rs fixtures/server-batch-manifest.tsv
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p pdfrust-cli batch_benchmark -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p pdfrust-cli --no-default-features -- benchmark-batch-native fixtures/generated --manifest fixtures/server-batch-manifest.tsv --include-family small --include-family mixed-size --include-family image-heavy --include-family repeated-resources --include-family vector-stress --repetitions 2 --max-workers 2 --max-in-flight-pixels 51200 --max-edge 160 --max-p95-ms 1000 --max-errors 0 --fail-on-budget --output target/server-batch-0133-benchmark.json
```
