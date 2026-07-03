# Complex Fill Coverage

Date: 2026-07-03
Issue: #25

## Summary

Complex nonzero and even-odd fills now stay on the exact scanline-cell coverage
route instead of falling back to per-pixel supersampling. The coverage path
accumulates globally sorted fill intervals across all active edges, splits rows
at local self-intersections, and reuses the same cell accumulator for
non-rectangular clip masks.

The change also widens miter-join stroke overlap bounds before the joined
outline route is selected. This keeps banded raster replay from skipping a
partial edge row when a miter extends beyond the simple stroke-radius padding.

## Benchmarked Fixture

The focused performance fixture is the `path-state` member of the operator
semantic snapshot corpus:

- `fixtures/generated/vector-paths.pdf`
- manifest: `fixtures/operator-semantic-snapshot-manifest.tsv`
- max edge: `160`
- iterations: `3`

Command shape:

```bash
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family path-state --max-edge 160 --iterations 3 --max-ms 10000 --max-output-bytes 1048576 --output target/issue-25-path-state-after.json
```

## Route Evidence

| Metric | Before | After |
| --- | ---: | ---: |
| Mean time | 14.721 ms | 0.423 ms |
| Coverage span calls | 2 | 2 |
| Coverage analytic edge pixels | 0 | 1534 |
| Coverage cell accumulator pixels | 0 | 2108 |
| Coverage sampled edge pixels | 1534 | 0 |
| Max partial alpha levels per call | 0 | 15 |

The post-change run rendered the fixture with `sampled_calls: 0` and
`coverage_sampled_edge_pixels: 0`, so the complex edge work stayed on the
scanline-cell route.

Artifacts:

- `target/issue-25-path-state-before.json`
- `target/issue-25-path-state-after.json`

## Visual Diff

Command:

```bash
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family path-state --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue-25-path-state-poppler.json
```

Result:

| Total | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 0 | 0 | 0 |

Path-state metrics:

- changed ratio: `0.137405`
- mean absolute error: `0.283`
- p95 channel delta: `1`
- max channel delta: `53`

The fixture remains inside the configured visual-diff policy thresholds.

## Regression Coverage

Focused coverage added or updated:

- complex overlapping and self-intersecting subpaths use scanline-cell coverage
  for both nonzero and even-odd fill rules;
- a row-center self-intersection renders through the scanline-cell route
  without sampled edge pixels;
- complex route counters now assert cell/analytic edge accounting and zero
  sampled edge pixels;
- miter stroke overlap bounds include the join extension in a clipped band;
- native banded joined-outline stroke replay preserves byte parity.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render
cargo test -p ferrugo-native native_banded_raster_should_match_single_target_for_joined_outline_stroke_paths -- --nocapture
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
