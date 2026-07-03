# Stroke Rect Outline Fast Path

Date: 2026-07-03

## Summary

This follow-up keeps the #26 production stroke-distance routes retired and adds
a direct coverage path for closed axis-aligned rectangle stroke outlines. The
target is the `vector-stress.pdf` cost left after route retirement: many `re S`
rectangle strokes were rendered as generic outline rings with inner holes, which
forced the scanline-cell fill path to process complex subpaths.

The new path detects solid butt-cap/miter-join closed axis-aligned rectangle
strokes and computes coverage from the outer rectangle minus the optional inner
rectangle. It still writes stroke outlines through coverage alpha; it does not
restore span, axis-span, row-bucket, or distance-predicate stroke routes.

## Route Audit

Commands:

```text
target/release/ferrugo trace-native fixtures/generated/dashed-stroke.pdf --max-edge 160 --output target/issue31-perf/dashed-stroke-trace.json
target/release/ferrugo trace-native fixtures/generated/line-caps.pdf --max-edge 160 --output target/issue31-perf/line-caps-trace.json
target/release/ferrugo trace-native fixtures/generated/line-joins.pdf --max-edge 160 --output target/issue31-perf/line-joins-trace.json
target/release/ferrugo trace-native fixtures/generated/vector-paths.pdf --max-edge 160 --output target/issue31-perf/vector-paths-trace.json
target/release/ferrugo trace-native fixtures/generated/vector-stress.pdf --max-edge 160 --output target/issue31-perf/vector-stress-ring-trace.json
```

| Fixture | Outline fills | Joined outlines | Axis-span calls | Axis-span joins | Span calls | Row-bucket calls |
|---|---:|---:|---:|---:|---:|---:|
| `dashed-stroke.pdf` | 10 | 0 | 0 | 0 | 0 | 0 |
| `line-caps.pdf` | 6 | 0 | 0 | 0 | 0 | 0 |
| `line-joins.pdf` | 6 | 6 | 0 | 0 | 0 | 0 |
| `vector-paths.pdf` | 2 | 2 | 0 | 0 | 0 | 0 |
| `vector-stress.pdf` | 66 | 22 | 0 | 0 | 0 | 0 |

`vector-stress.pdf` now reports only `2` generic fill coverage-span calls for
the remaining non-rect outline work, down from `22` before this change.

## Visual Diff

Commands used the same Poppler thresholds as the final #26 cleanup, including
the wider existing `vector-stress.pdf` threshold.

| Fixture | Status | Changed pixels | Changed ratio | MAE | p95 | Max |
|---|---|---:|---:|---:|---:|---:|
| `dashed-stroke.pdf` | exact | 0 | 0.000000 | 0.000 | 0 | 0 |
| `line-caps.pdf` | accepted drift | 13 | 0.000903 | 0.033 | 0 | 72 |
| `line-joins.pdf` | accepted drift | 21 | 0.001458 | 0.170 | 0 | 255 |
| `vector-paths.pdf` | accepted drift | 2880 | 0.137405 | 0.283 | 1 | 53 |
| `vector-stress.pdf` | accepted drift | 5559 | 0.289531 | 12.362 | 103 | 160 |

## Benchmark

Command:

```text
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 200 --max-ms 10000 --max-output-bytes 1048576 --output target/issue31-perf/vector-stress-ring-benchmark.json
```

Result:

| Build | Mean |
|---|---:|
| Current outline route before this follow-up | 2.576 ms |
| Rect outline fast path | 2.195 ms |
| Earlier distance-route baseline from #44 | 0.472 ms |

This is an improvement over the current outline-only route, not full
neutrality against the retired distance-predicate route. The #26 close review
still needs to accept the remaining cost or continue with a separate
optimization.
