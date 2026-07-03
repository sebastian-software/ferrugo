# Fill Edge Y-Break Cache

Date: 2026-07-03
Issue: #49

## Summary

`FillEdgeCoverage` now caches edge-intersection y-breaks per draw path instead
of recomputing the pairwise edge-intersection scan for every raster row. Row
coverage extracts the relevant y-break subrange with binary search, then keeps
the existing scanline-cell slab rasterizer unchanged.

The cache keeps the existing 256-edge quadratic-scan cap. Paths above that cap
fall back to the old row-local scan so behavior stays conservative for very
large paths.

The PR #48 generated-stroke hint is still useful, but the old
`skip_intersection_y_breaks` to `requires_intersection_y_breaks` run plumbing is
gone. Generated stroke outlines now seed the per-path cache as known-empty when
their geometry proves no interior edge-intersection y-breaks are needed.

## Performance

Same host, release binary, `--max-edge 160`.

| Fixture | Build | Iterations | Mean |
| --- | --- | ---: | ---: |
| `vector-stress.pdf` | `main` (`c374d2b`) | 500 | 2.046 ms |
| `vector-stress.pdf` | `main` (`c374d2b`) | 500 | 2.046 ms |
| `vector-stress.pdf` | this branch | 500 | 1.806 ms |
| `vector-stress.pdf` | this branch | 500 | 1.809 ms |
| `vector-paths.pdf` (`path-state`) | `main` (`c374d2b`) | 100 | 0.207 ms |
| `vector-paths.pdf` (`path-state`) | this branch | 100 | 0.205 ms |

Artifacts:

- Baseline: `target/issue49-main-vector-stress-1.json`,
  `target/issue49-main-vector-stress-2.json`,
  `target/issue49-main-path-state.json`.
- Branch: `target/issue49-branch-vector-stress-1.json`,
  `target/issue49-branch-vector-stress-2.json`,
  `target/issue49-branch-path-state.json`.

Honest accounting: this is a stable improvement on `vector-stress.pdf`
(`2.046 ms` to `1.806/1.809 ms`) and neutral on the fill-heavy `path-state`
fixture. It does not reach the earlier unsound `~1.49 ms` result or the retired
distance-route baseline. A measured experiment that also reused the cached edge
list for slab intersections regressed `vector-stress.pdf` to about `1.90 ms`, so
that change was not kept.

## Route And Visual Evidence

Final `trace-native` artifacts:

- `target/issue49-branch-vector-stress-trace.json`
- `target/issue49-branch-vector-paths-trace.json`

Route counters stay on the outline path and do not bring back retired stroke
routes:

| Fixture | outline fill | outline joined | axis span | span | row bucket |
| --- | ---: | ---: | ---: | ---: | ---: |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 |

Poppler visual-diff artifacts:

| Fixture | Status | Changed pixels | Changed ratio | MAE | p95 | Max delta |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `vector-stress.pdf` | accepted drift | 5559 | 0.289531 | 12.362 | 103 | 160 |
| `vector-paths.pdf` | accepted drift | 2880 | 0.137405 | 0.283 | 1 | 53 |

Artifacts:

- `target/issue49-vector-stress-poppler.json`
- `target/issue49-path-state-poppler.json`

The native banded-vs-single byte-parity matrix was rerun and stayed green.

## Validation

Commands run:

```sh
cargo fmt --check
cargo test -p ferrugo-render fill_edge_intersection_y_break_cache --no-default-features
cargo test -p ferrugo-render round_polyline_stroke_outline_should_mark_only_simple_y_break_caches_empty --no-default-features
cargo test -p ferrugo-render path_rasterizer_should_draw_generated_vector_stress_fixture --no-default-features -- --nocapture
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issue49-branch-vector-stress-1.json
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 500 --max-ms 1000000 --max-output-bytes 1048576 --output target/issue49-branch-vector-stress-2.json
target/release/ferrugo benchmark-native fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family path-state --max-edge 160 --iterations 100 --max-ms 1000000 --max-output-bytes 1048576 --output target/issue49-branch-path-state.json
target/release/ferrugo trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue49-branch-vector-stress-trace.json
target/release/ferrugo trace-native fixtures/generated/vector-paths.pdf --max-edge 160 --output target/issue49-branch-vector-paths-trace.json
target/release/ferrugo visual-diff-poppler fixtures/generated/vector-stress.pdf --max-edge 160 --max-mae 60.0 --max-p95 190 --max-changed-ratio 0.55 --timeout 30 --output target/issue49-vector-stress-poppler.json
target/release/ferrugo visual-diff-poppler fixtures/generated --manifest fixtures/operator-semantic-snapshot-manifest.tsv --include-family path-state --max-edge 160 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue49-path-state-poppler.json
cargo test -p ferrugo-render --no-default-features
cargo test -p ferrugo-native native_banded_raster_should_match_single_target_output --no-default-features -- --nocapture
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```
