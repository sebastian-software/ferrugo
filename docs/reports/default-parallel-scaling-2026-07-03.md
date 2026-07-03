# Default-Budget Parallel Band Scaling

Date: 2026-07-03
Issue: #29

## Summary

This slice adds opt-in default-budget parallel band profiles so worker scaling
can be measured on real generated fixtures that exceed the default banding
threshold. It does not make parallel replay the default.

New native profiles:

| Profile | Render budgets | Max band workers |
| --- | --- | ---: |
| `default-parallel` | default | 2 |
| `default-parallel-2` | default | 2 |
| `default-parallel-4` | default | 4 |
| `default-parallel-8` | default | 8 |

`default` remains serial with `max_raster_band_workers = 1`.
`low-memory-parallel*` keeps the constrained low-memory budgets.

## Corpus Scaling Evidence

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated --native-profile default --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue29-default-parallel/default.json
target/release/ferrugo benchmark-native fixtures/generated --native-profile default-parallel --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue29-default-parallel/default-parallel-2.json
target/release/ferrugo benchmark-native fixtures/generated --native-profile default-parallel-4 --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue29-default-parallel/default-parallel-4.json
target/release/ferrugo benchmark-native fixtures/generated --native-profile default-parallel-8 --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue29-default-parallel/default-parallel-8.json
```

Artifacts:

- `target/issue29-default-parallel/default.json`
- `target/issue29-default-parallel/default-parallel-2.json`
- `target/issue29-default-parallel/default-parallel-4.json`
- `target/issue29-default-parallel/default-parallel-8.json`

All four runs rendered the same corpus shape: 233 total fixtures, 217 native
renders, 12 fallback-required fixtures, 4 errors, and 16 budget failures. Nine
native-rendered fixtures crossed the default banding threshold.

| Profile | Banded fixtures | Max observed workers | Native mean-ms sum | RSS high-water bytes |
| --- | ---: | ---: | ---: | ---: |
| `default` | 9 | 1 | 137.980 | 17,563,648 |
| `default-parallel` | 9 | 2 | 139.089 | 21,643,264 |
| `default-parallel-4` | 9 | 4 | 90.244 | 18,726,912 |
| `default-parallel-8` | 9 | 8 | 77.526 | 19,070,976 |

The 2-worker run is essentially neutral/slower on the full generated corpus,
while 4 and 8 workers improve total native render time for the same fixture
set. RSS high-water increases versus serial, which is the expected tradeoff:
parallel replay keeps multiple active band targets live.

## Eight-Worker Fixture Spread

`default-parallel-8` reached eight workers on seven real fixtures and capped to
the available band count on two fixtures.

| Fixture | Mean time | Bands | Workers | Active target peak bytes | Estimated peak raster bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| `academic-publisher-first-page.pdf` | 0.899 ms | 8 | 8 | 691,200 | 1,382,400 |
| `browser-chromium-article-print.pdf` | 0.486 ms | 8 | 8 | 662,400 | 1,324,800 |
| `engineering-large-transform-detail.pdf` | 9.427 ms | 11 | 8 | 2,097,152 | 4,894,720 |
| `high-dpi-preview-fidelity.pdf` | 0.587 ms | 6 | 6 | 691,200 | 1,382,400 |
| `image-heavy-repeated-xobject-report.pdf` | 1.550 ms | 8 | 8 | 768,000 | 1,536,000 |
| `layout-columns-footnotes-table-stress.pdf` | 0.972 ms | 9 | 8 | 819,200 | 1,651,200 |
| `page-size-letter.pdf` | 0.598 ms | 13 | 8 | 1,253,376 | 3,192,192 |
| `scientific-two-column-paper.pdf` | 0.821 ms | 8 | 8 | 691,200 | 1,382,400 |
| `technical-large-coordinate-plan.pdf` | 6.600 ms | 10 | 8 | 2,097,152 | 4,612,096 |

## Policy

Keep parallel band replay opt-in.

Reasons:

- Serial `default` remains the broad, predictable production default.
- The 2-worker default-budget profile is not a corpus-wide win in this run.
- Higher worker caps improve throughput on the banded fixture spread but spend
  more active raster memory and raise process RSS high-water.
- Document-session Type3 render-cache reuse still intentionally disables
  parallel replay. The policy is to keep cached document-session rendering
  serial until a per-thread Type3 render cache or merge strategy has separate
  byte-parity and cache-hit evidence.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo benchmark_config_should_accept_low_memory_parallel_native_profile -- --nocapture
cargo test -p ferrugo
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
