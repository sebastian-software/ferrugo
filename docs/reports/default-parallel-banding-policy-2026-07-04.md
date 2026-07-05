# Default Parallel Banding Policy - 2026-07-04

## Scope

Issue #112 promotes large-page native band replay from opt-in profiles to the
default desktop profile while preserving the strict low-memory serial profile.

The new default policy:

- keeps the existing `64`-row band size and `160000`-pixel threshold;
- caps default parallel band replay at `min(available_parallelism, 4)`;
- keeps `low-memory` serial with one worker;
- keeps explicit `default-parallel-*` and `low-memory-parallel-*` profile caps
  for tuning comparisons;
- allows parallel band replay when a document-session Type3 render cache exists.
  Parallel workers create per-band Type3 template caches and run without a
  Type3 render cache (the session render cache is not shared across threads),
  so Type3 glyph rendering is not memoized under parallel banding. Correction
  2026-07-05: the original wording claimed local per-band render caches; the
  code passes none. Restoring per-worker render-cache memoization is tracked
  in the post-0.4.0 polish backlog.

On the benchmark host, `available_parallelism` was 20, so the default cap was 4.

## Benchmark Command

Baseline was run from `main` at `48a782e`. The after run was run from
`codex/issue-112-default-parallel-banding`.

```sh
cargo run -p ferrugo --release --no-default-features -- benchmark-native \
  fixtures/generated/high-dpi-preview-fidelity.pdf \
  --native-profile default \
  --max-edge 1024 \
  --iterations 10 \
  --max-ms 10000 \
  --max-output-bytes 8388608
```

## Results

| Run | Profile | Output | Bands | Workers | Mean ms | Active target bytes | Estimated peak raster bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Before | `default` | 480x360 | 6 | 1 | 0.724 | 122880 | 814080 |
| After | `default` | 480x360 | 6 | 4 | 0.572 | 491520 | 1182720 |

The default profile now uses four band workers on this high-DPI fixture and
improves mean latency by 21.0%. The tradeoff is intentional: active target bytes
rise from one band target to four band targets, while the renderer remains below
the default page/output budgets.

The scanner fixture at `max-edge=450` remains below the default threshold
(`140800` pixels), so it correctly stays single-target under the default profile.

## Validation

```sh
scripts/check_scheduler_tuning_matrix.sh
bash scripts/check_low_end_reliability_matrix.sh
cargo test -p ferrugo-native default_profile_should_band_only_above_pixel_threshold -- --nocapture
cargo test -p ferrugo-native native_backend_should_expose_memory_diagnostics -- --nocapture
cargo test -p ferrugo-native native_low_memory_profile_should_expose_tighter_memory_diagnostics -- --nocapture
cargo test -p ferrugo-native low_memory_parallel_render_should_match_serial_type3_fixtures -- --nocapture
cargo test -p ferrugo benchmark_config_should_accept_low_memory_parallel_native_profile -- --nocapture
cargo fmt --check
cargo check -p ferrugo --no-default-features
cargo test -p ferrugo-native
cargo test -p ferrugo
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```

All commands passed locally.
