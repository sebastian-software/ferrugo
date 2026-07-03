# Default Banded Replay Rollout

Date: 2026-07-03
Issue: #28

## Summary

This slice enables serial banded replay in the default native profile for
supported pages at or above `160,000` raster pixels. Smaller default-profile
renders stay on the legacy single-target path. The threshold avoids applying
banding to small pages where the extra replay overhead does not buy meaningful
memory relief, while allowing larger supported pages to reduce the active
raster target.

Low-memory profiles keep their existing behavior: `low-memory` and
`low-memory-parallel` still band with a zero-pixel threshold.

## Default Profile Policy

Default native limits now report:

| Limit | Value |
| --- | ---: |
| `max_raster_band_rows` | 64 |
| `min_raster_band_pixels` | 160,000 |
| `max_raster_band_workers` | 1 |

The default rollout is serial only. Parallel band replay remains opt-in through
`low-memory-parallel*`.

## Corpus Coverage

Command:

```bash
target/release/ferrugo benchmark-native fixtures/generated --native-profile default --max-edge 1024 --iterations 1 --max-ms 10000 --max-output-bytes 8388608 --output target/issue28-default/default-corpus-coverage-rss.json
```

Artifact:

- `target/issue28-default/default-corpus-coverage-rss.json`

Summary:

| Metric | Count |
| --- | ---: |
| Total fixtures | 233 |
| Native rendered | 217 |
| Fallback required | 12 |
| Errors | 4 |
| Budget failures | 16 |
| Native rendered with banding | 9 |
| Native rendered single-target | 208 |

The RSS-enabled corpus run reported `18,595,840` bytes process RSS high-water.

Above-threshold default-banded fixtures:

| Fixture | Output | Bands | Active target peak bytes | Active target reduction |
| --- | ---: | ---: | ---: | ---: |
| `academic-publisher-first-page.pdf` | 360x480 | 8 | 92,160 | 866 permille |
| `browser-chromium-article-print.pdf` | 360x460 | 8 | 92,160 | 860 permille |
| `engineering-large-transform-detail.pdf` | 1024x683 | 11 | 262,144 | 906 permille |
| `high-dpi-preview-fidelity.pdf` | 480x360 | 6 | 122,880 | 822 permille |
| `image-heavy-repeated-xobject-report.pdf` | 400x480 | 8 | 102,400 | 866 permille |
| `layout-columns-footnotes-table-stress.pdf` | 400x520 | 9 | 102,400 | 876 permille |
| `page-size-letter.pdf` | 612x792 | 13 | 156,672 | 919 permille |
| `scientific-two-column-paper.pdf` | 360x480 | 8 | 92,160 | 866 permille |
| `technical-large-coordinate-plan.pdf` | 1024x614 | 10 | 262,144 | 895 permille |

## Before/After Evidence

Commands:

```bash
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/high-dpi-preview-fidelity.pdf --native-profile default --max-edge 1024 --iterations 5 --max-ms 10000 --max-output-bytes 8388608 --output target/issue28-default/before-default-high-dpi.json
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/high-dpi-preview-fidelity.pdf --native-profile default --max-edge 1024 --iterations 5 --max-ms 10000 --max-output-bytes 8388608 --output target/issue28-default/after-threshold-default-high-dpi.json
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile default --max-edge 440 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue28-default/before-default-scanner.json
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/scanner-large-image-budget.pdf --native-profile default --max-edge 440 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue28-default/after-threshold-default-scanner.json
```

Artifacts:

- `target/issue28-default/before-default-high-dpi.json`
- `target/issue28-default/after-threshold-default-high-dpi.json`
- `target/issue28-default/before-default-scanner.json`
- `target/issue28-default/after-threshold-default-scanner.json`

| Fixture/profile | Mean time | Bands | Active target peak bytes | Estimated peak raster bytes | RSS high-water bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| high-DPI before | 0.683 ms | 1 | 691,200 | 691,200 | 6,864,896 |
| high-DPI after | 0.888 ms | 6 | 122,880 | 814,080 | 5,636,096 |
| scanner before | 0.607 ms | 1 | 563,200 | 563,200 | 6,717,440 |
| scanner after | 0.625 ms | 1 | 563,200 | 563,200 | 7,307,264 |

The high-DPI fixture crosses the threshold and shows an active-target reduction
from `691,200` to `122,880` bytes plus lower sampled process RSS high-water in
this run. The scanner fixture stays below the threshold, so the default raster
shape remains single-target; its RSS movement is treated as process noise, not
as a banding claim.

## Parity And Determinism

The existing banded replay matrix remains the byte-parity guard for supported
surfaces, including Type3 fixtures added in
`docs/reports/banded-type3-coverage-2026-07-03.md`. This slice adds
`default_profile_should_band_only_above_pixel_threshold`, which verifies that
the default profile keeps the scanner-sized page single-target and bands the
above-threshold high-DPI fixture with the expected 64-row shape.

## Remaining #28 Work

This is a thresholded default-profile rollout, not a claim that every generic
surface now bands. The coverage report shows that 208 native-rendered fixtures
still use single-target replay, either because they are below the threshold or
because the conservative band-support guards keep them on the legacy path.

The remaining #28 work is to keep widening the supported generic surfaces and
raise the default coverage share when parity evidence supports it.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-native default_profile_should_band_only_above_pixel_threshold -- --nocapture
cargo test -p ferrugo-native native_banded_raster_should_match_single_target_output -- --nocapture
cargo test -p ferrugo benchmark_native_should_report_fill_route_summary -- --nocapture
cargo test -p ferrugo-native
cargo test -p ferrugo
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
