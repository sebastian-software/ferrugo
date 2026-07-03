# Band-Parallel Type3 Coverage

Date: 2026-07-03
Issue: #29

## Summary

This slice adds Type3 coverage to the low-memory parallel band replay parity
suite. It does not choose a default policy, widen the worker cap, or resolve
the document-session Type3 render-cache vs. parallelism boundary. The goal is
to prove that request-local `low-memory-parallel` band replay is byte-identical
to serial low-memory replay for Type3 fixtures and that the scheduler actually
uses two band workers on a Type3 page.

## Parallel Byte-Parity Coverage

Added test:

```bash
cargo test -p ferrugo-native low_memory_parallel_render_should_match_serial_type3_fixtures -- --nocapture
```

Fixtures covered:

| Fixture | Output | Bands | Workers | Active target peak bytes |
| --- | ---: | ---: | ---: | ---: |
| `subset-type3-repeated-charprocs.pdf` | 260x120 | 2 | 2 | 124,800 |
| `type3-barcode-font.pdf` | 220x160 | 3 | 2 | 112,640 |

Both cases render once through `NativeBackend::low_memory()` and once through
`NativeBackend::low_memory_parallel()`, then assert exact RGBA byte parity. The
repeated-CharProc case covers Type3 template reuse shape; the barcode case is
tall enough for two active band workers to reduce the active raster target
below the full page.

## Benchmark Evidence

Focused fixture:

- `fixtures/generated/type3-barcode-font.pdf`
- max edge: `220`
- iterations: `10`

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/type3-barcode-font.pdf --native-profile low-memory --max-edge 220 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue-29-type3-barcode-serial-benchmark.json
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/type3-barcode-font.pdf --native-profile low-memory-parallel --max-edge 220 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue-29-type3-barcode-parallel-benchmark.json
```

Artifacts:

- `target/issue-29-type3-barcode-serial-benchmark.json`
- `target/issue-29-type3-barcode-parallel-benchmark.json`

| Metric | Serial low-memory | Parallel low-memory |
| --- | ---: | ---: |
| Mean time | 0.124 ms | 0.233 ms |
| Output bytes | 140,800 | 140,800 |
| Bands | 3 | 3 |
| Workers | 1 | 2 |
| Active target peak bytes | 56,320 | 112,640 |
| Estimated peak raster bytes | 197,120 | 253,440 |
| Active target reduction | 600 permille | 200 permille |

The parallel path is slower on this small Type3 fixture, which is expected
worker overhead rather than a default-on signal. The evidence is useful as a
correctness and scheduler-shape guard only.

## Remaining #29 Work

The full #29 acceptance still requires:

- broader parallel byte-parity coverage for text, shading, transparency, and
  more Type3 shapes;
- worker scaling measurements across 2/4/8 workers and a fixture spread;
- RSS high-water measurements per worker count;
- an explicit default policy decision;
- a decision or implementation for document-session Type3 render-cache reuse
  alongside parallel band replay.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-native low_memory_parallel_render_should_match_serial_type3_fixtures -- --nocapture
cargo test -p ferrugo-native
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
