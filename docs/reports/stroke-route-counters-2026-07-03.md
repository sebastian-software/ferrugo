# Stroke Route Counters

Date: 2026-07-03
Issue: #26

## Summary

This slice does not complete the full stroke outliner retirement. It widens the
existing joined-outline eligibility to include round joins when the conservative
joined-outline guard already selects that route, and it adds explicit runtime
route counters for the axis-span stroke path.

Before this change, `line-joins.pdf` showed six axis-span routed stroke items in
the shape summary, but the runtime stroke route summary still reported zero
production stroke-route calls. The new counters expose those remaining
axis-span joined strokes directly so the larger #26 retirement work can measure
them as production distance/span work instead of treating them as an invisible
gap.

During this work, forcing the small axis-aligned joined strokes in
`line-joins.pdf` through the joined-outline route regressed the local benchmark
from `0.257 ms` to `1.032 ms`. That experiment was not shipped. The committed
change keeps the existing axis-span routing order and threshold for those small
joined strokes.

## Route Evidence

Fixture:

- `fixtures/generated/line-joins.pdf`
- max edge: `120`

Artifacts:

- `target/issue-26-line-joins-before.json`
- `target/issue-26-line-joins-after.json`

| Runtime stroke route metric | Before | After |
| --- | ---: | ---: |
| Outline fill calls | 0 | 0 |
| Joined outline calls | 0 | 0 |
| Axis-span calls | not reported | 6 |
| Axis-span joined calls | not reported | 6 |
| Axis-span coverage spans | not reported | 408 |
| Axis-span raster spans | not reported | 408 |
| Span-covered calls | 0 | 0 |
| Row-bucket range calls | 0 | 0 |

The shape summary is unchanged for the fixture:

| Shape metric | Before | After |
| --- | ---: | ---: |
| Stroked items | 6 | 6 |
| Axis-aligned items | 6 | 6 |
| Axis-span routed items | 6 | 6 |
| Axis-span coverage spans | 408 | 408 |
| Axis-span raster spans | 408 | 408 |
| Generic stroke fallback items | 0 | 0 |

## Benchmark Evidence

Command shape:

```bash
cargo run --release -p ferrugo --no-default-features -- benchmark-native fixtures/generated/line-joins.pdf --max-edge 120 --iterations 10 --max-ms 10000 --max-output-bytes 1048576 --output target/issue-26-line-joins-after-benchmark-rerun.json
```

| Fixture | Baseline | After |
| --- | ---: | ---: |
| `line-joins.pdf` mean time | 0.207 ms | 0.165 ms |

Artifacts:

- baseline: `target/issue-26-line-joins-before-benchmark-rerun.json`
- after: `target/issue-26-line-joins-after-benchmark-rerun.json`

The benchmark path does not request runtime stroke route summaries, so the new
counters are inactive during normal rendering unless trace collection provides a
route sink.

## Visual Diff

Command shape:

```bash
cargo run --release -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated/line-joins.pdf --max-edge 120 --max-mae 6.0 --max-p95 24 --max-changed-ratio 0.18 --timeout 30 --output target/issue-26-line-joins-poppler.json
```

Result:

| Total | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 0 | 0 | 0 |

Metrics:

- changed pixels: `20`
- changed ratio: `0.001389`
- mean absolute error: `0.178`
- p95 channel delta: `0`
- max channel delta: `255`
- native nonwhite pixels: `1427`
- reference nonwhite pixels: `1433`

The fixture remains inside the configured visual-diff policy thresholds.

## Regression Coverage

Focused coverage added or updated:

- non-axis miter, bevel, and round joins now all record joined-outline fill
  calls when the existing conservative joined-outline guard selects that path;
- axis-aligned joined strokes now record axis-span calls, joined axis-span
  calls, coverage spans, and raster spans;
- native trace JSON exposes the new axis-span stroke route fields.

## Remaining #26 Work

The full #26 acceptance still requires the larger stroke outliner pass:

- round and square caps;
- dash phase and dash-collapse policy;
- closed and multi-subpath strokes;
- degenerate segment handling;
- corpus-level route evidence showing zero production distance/span calls;
- fixture parity or documented drift across `line-caps`, `line-joins`,
  `dashed-*`, `vector-stress`, and `vector-paths`;
- release-profile evidence that the distance-predicate symbols disappear from
  hot profiles.

## Validation

```bash
cargo fmt --check
cargo test -p ferrugo-render stroke_raster_route_summary_should_count -- --nocapture
cargo test -p ferrugo native_trace_json_should_omit_document_bytes_and_bound_events -- --nocapture
cargo test -p ferrugo-render
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
