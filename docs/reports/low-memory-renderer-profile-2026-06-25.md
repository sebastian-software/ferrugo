# Low-Memory Renderer Profile 2026-06-25

Milestone: 0131.

## Decision

The native renderer now has an explicit low-memory profile for constrained
thumbnail workloads. The default backend remains desktop-oriented, while
`NativeBackend::low_memory()` and `NativeBackend::with_render_limits(...)`
allow callers and tests to choose tighter page raster, decoded image,
display-list, font/cache, vector, pattern, and transparency-intermediate
budgets.

The focused low-memory corpus renders 5/5 fixtures under the documented
profile with no native fallbacks, errors, or benchmark budget failures.

## Profile Budgets

The low-memory profile currently uses these thumbnail-oriented caps:

| Budget | Limit |
| --- | ---: |
| Page raster pixels | 147456 |
| Decoded bytes per image | 12582912 |
| Decoded image bytes per page | 25165824 |
| ICC profile bytes | 262144 |
| ICC transform workspace bytes | 32768 |
| ICC transform cache entries | 8 |
| Embedded font program bytes | 4194304 |
| ToUnicode CMap bytes | 262144 |
| Text run bytes | 16384 |
| Display items | 2048 |
| Font fallback cache entries | 32 |
| Transparency group pixels | 262144 |
| Flattened path segments | 16384 |
| Pattern tiles | 16384 |
| Pattern cell cache entries | 8 |

Temporary spooling remains disabled in this slice.

## Native Gate Evidence

Artifact: `target/low-memory-0131-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `common` | 2 | 2 | 0 | 0 |
| `scan` | 1 | 1 | 0 | 0 |
| `vector-stress` | 1 | 1 | 0 | 0 |
| `repeated-resources` | 1 | 1 | 0 | 0 |
| **Total** | **5** | **5** | **0** | **0** |

The gate uses `--native-profile low-memory`, `--max-edge 160`, and
`--fail-on-fallback`.

## Benchmark Evidence

Artifact: `target/low-memory-0131-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `common` | 2 | 2 | 15.487 | 30.416 | 0 |
| `scan` | 1 | 1 | 40.464 | 40.464 | 0 |
| `vector-stress` | 1 | 1 | 188.005 | 188.005 | 0 |
| `repeated-resources` | 1 | 1 | 15.970 | 15.970 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Failure Behavior

Unit coverage intentionally sets `max_page_pixels` to `1` and verifies that the
native renderer returns a typed unsupported error in the
`renderer.memory-budget` bucket instead of attempting an oversized allocation.
The same diagnostics path exposes the active limits through
`NativeBackend::memory_diagnostics()` and CLI JSON output.

## Follow-Up Backlog

- Add allocator or resident-set measurement only after a stable measurement
  harness exists.
- Revisit Form XObject resource collection if future nested-form documents need
  stricter decode-time budget propagation before display-list construction.
- Keep the low-memory profile thumbnail-oriented; high-fidelity desktop
  defaults should remain separate.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-native/src/lib.rs crates/pdfrust-cli/src/main.rs fixtures/low-memory-profile-manifest.tsv
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p pdfrust-native low_memory -- --nocapture
cargo test -p pdfrust-cli benchmark_config_should_accept_low_memory_native_profile -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family scan --include-family vector-stress --include-family repeated-resources --fail-on-fallback --max-edge 160 --native-profile low-memory --output target/low-memory-0131-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/low-memory-profile-manifest.tsv --include-family common --include-family scan --include-family vector-stress --include-family repeated-resources --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --native-profile low-memory --output target/low-memory-0131-benchmark.json
```
