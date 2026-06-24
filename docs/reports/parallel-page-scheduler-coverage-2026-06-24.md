# Parallel Page Scheduler Coverage 2026-06-24

This report records milestone 0077 coverage for bounded parallel multi-page
rendering in the Rust-native backend.

## Implemented Slice

- Added `ParallelRenderOptions` for explicit worker and in-flight pixel budgets.
- Added `ParallelRenderResult` with ordered page results and the effective
  worker count after applying memory limits.
- Added `render_pages_parallel`, a scoped-thread scheduler that:
  - loads the input source once;
  - shares borrowed PDF bytes across page workers without cloning document bytes;
  - renders requested pages in bounded batches;
  - preserves requested result order;
  - returns the first requested page error in a started batch after joining
    workers;
  - backs off worker count when the in-flight pixel budget is tight.
- Added native-backend tests for order stability, byte-for-byte parity against
  sequential renders, memory-budget backoff, memory-budget failure, and
  deterministic page-error behavior.

## Scheduler Policy

The scheduler parallelizes across pages only. It does not add an async runtime
and does not parallelize work within a single page. The current implementation
shares the input bytes, but each worker still builds its own render state for
the requested page. This keeps ownership simple and avoids shared mutable caches
until cache synchronization has stronger benchmark evidence.

The memory policy is conservative: each worker is budgeted as if it could render
up to `max_edge * max_edge` pixels. If `max_in_flight_pixels` cannot cover even
one page, scheduling fails with the existing `renderer.memory-budget`
unsupported bucket. If it can cover fewer pages than `max_workers`, the
effective worker count is reduced.

## Validation

```text
cargo test -p pdfrust-native native_parallel_renderer -- --nocapture
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
/usr/bin/time -l cargo test -p pdfrust-native native_parallel_renderer_should_match_sequential_page_outputs -- --nocapture
```

All commands completed successfully.

The full test suite reported 68 `pdfrust-native` tests passing, including five
parallel scheduler tests. The targeted scheduler timing command reported:

```text
test tests::native_parallel_renderer_should_match_sequential_page_outputs ... ok
0.68 real
25444352 maximum resident set size
8372728 peak memory footprint
```

## Remaining Limits

- There is no CLI subcommand for multi-page parallel rendering yet.
- Workers currently share input bytes but not a parsed object table or resource
  caches.
- The scheduler uses a conservative pixel-budget estimate, not measured
  per-page allocations.
- Deeper throughput benchmarks should move into the renderer benchmark suite in
  milestone 0078.
