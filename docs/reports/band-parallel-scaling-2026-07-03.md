# Band-Parallel Worker Scaling

Date: 2026-07-03
Issue: #29

## Summary

This slice exposes bounded 2/4/8 worker caps for the low-memory parallel band
profile, extends parallel byte-parity coverage beyond Type3, and records RSS
high-water benchmark evidence for the large scanner fixture. It does not make
parallel replay the default. The default remains opt-in because the higher
worker caps trade active raster target bytes for latency, and the
document-session Type3 render-cache boundary still needs a final policy.

## CLI Profiles

`benchmark-native` now accepts these native profiles:

| Profile | Max band workers |
| --- | ---: |
| `low-memory-parallel` | 2 |
| `low-memory-parallel-2` | 2 |
| `low-memory-parallel-4` | 4 |
| `low-memory-parallel-8` | 8 |

The worker count remains a cap. A page with fewer raster bands uses fewer
workers.

## Parallel Byte-Parity Coverage

Added focused test:

```bash
cargo test -p ferrugo-native low_memory_parallel_render_should_match_serial_extended_fixture_set -- --nocapture
```

Fixtures covered across 2/4/8 worker caps:

| Fixture | Coverage surface |
| --- | --- |
| `text-page.pdf` | text/list replay |
| `axial-gradient.pdf` | shading replay |
| `transparency-alpha.pdf` | transparency replay |

Each case renders once with `NativeBackend::low_memory()` and then with
`NativeBackend::low_memory_parallel_with_workers(2)`,
`NativeBackend::low_memory_parallel_with_workers(4)`, and
`NativeBackend::low_memory_parallel_with_workers(8)`. The test asserts exact
RGBA byte parity and verifies that the measured worker count is
`min(requested_workers, bands)`.

Added synthetic scheduler test:

```bash
cargo test -p ferrugo-native low_memory_parallel_render_should_scale_to_eight_workers_for_large_banded_page -- --nocapture
```

The synthetic 640x640 filled page creates 10 raster bands. The 8-worker profile
uses 8 workers, remains byte-identical to serial low-memory banding, and reports
an active target peak of `640 * 64 * 4 * 8 = 1,310,720` bytes.

## Benchmark Evidence

Focused fixture:

- `fixtures/generated/scanner-large-image-budget.pdf`
- max edge: `450`
- iterations: `10`
- output bytes: `563,200`

Commands:

```bash
target/release/ferrugo benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile low-memory --max-edge 450 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue29-scaling/scanner450-low-memory.json
target/release/ferrugo benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile low-memory-parallel --max-edge 450 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue29-scaling/scanner450-low-memory-parallel-2.json
target/release/ferrugo benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile low-memory-parallel-4 --max-edge 450 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue29-scaling/scanner450-low-memory-parallel-4.json
target/release/ferrugo benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile low-memory-parallel-8 --max-edge 450 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue29-scaling/scanner450-low-memory-parallel-8.json
```

Artifacts:

- `target/issue29-scaling/scanner450-low-memory.json`
- `target/issue29-scaling/scanner450-low-memory-parallel-2.json`
- `target/issue29-scaling/scanner450-low-memory-parallel-4.json`
- `target/issue29-scaling/scanner450-low-memory-parallel-8.json`

| Profile | Mean time | Bands | Workers | Active target peak bytes | Estimated peak raster bytes | RSS high-water bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `low-memory` | 4.882 ms | 7 | 1 | 81,920 | 645,120 | 6,995,968 |
| `low-memory-parallel` | 3.362 ms | 7 | 2 | 163,840 | 727,040 | 7,143,424 |
| `low-memory-parallel-4` | 2.407 ms | 7 | 4 | 327,680 | 890,880 | 6,553,600 |
| `low-memory-parallel-8` | 1.514 ms | 7 | 7 | 563,200 | 1,126,400 | 7,536,640 |

The generated scanner fixture has seven bands, so the 8-worker profile is
capped at seven real workers. That is still useful scaling evidence for the
cap behavior: latency improves as more bands replay concurrently, while active
target bytes rise until the 7-worker case equals the full output buffer.

## Default Policy

Keep `low-memory-parallel` opt-in for now.

Reasons:

- The small Type3 benchmark in
  `docs/reports/band-parallel-type3-coverage-2026-07-03.md` is slower in
  parallel mode, so this is not a universal default win.
- Higher worker caps intentionally spend more active target bytes; the
  7-worker scanner case reaches `563,200` active target bytes.
- The Type3 document-session render-cache path still disables parallel replay.
  The default should not change until that cache boundary is decided.

## Remaining #29 Work

The full #29 acceptance still requires:

- a corpus-level fixture spread with real-PDF 2/4/8 worker measurements where
  pages naturally expose enough bands;
- a final decision or implementation for document-session Type3 render-cache
  reuse alongside parallel band replay.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo benchmark_config_should_accept_low_memory_parallel_native_profile -- --nocapture
cargo test -p ferrugo-native low_memory_parallel_render_should_match_serial_extended_fixture_set -- --nocapture
cargo test -p ferrugo-native low_memory_parallel_render_should_scale_to_eight_workers_for_large_banded_page -- --nocapture
cargo test -p ferrugo-native
cargo test -p ferrugo
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
