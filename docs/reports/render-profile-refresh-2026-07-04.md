# Render Profile Refresh

Date: 2026-07-04
Issue: #100

## Summary

This refresh re-profiles the vector and text-adjacent render families after the
stroke-to-fill, coverage-fill, adaptive flattening, and default-banding work.
The active optimization backlog should no longer use the old
`rasterize_row_bucketed_stroke_ranges` or
`rasterize_span_covered_stroke_ranges` entries as current hot spots. Those
names remain useful only as historical breadcrumbs in older profiling notes.

Current evidence points to this order:

1. #113 partial-coverage edge pixels and coverage-fill batching for
   vector/report work, especially `vector-stress.pdf` and
   `technical-hatch-clipping.pdf`.
2. #110 SIMD span and row blitters, scoped first to the measured coverage,
   shading, text-rectangle, and normal source-over rows rather than broad
   renderer-wide SIMD.
3. #111 font-face and glyph cache work, kept behind session/request cache
   policy and measured first on office/text fixtures.
4. #112 parallel banding defaults, deferred for this fixture set because every
   requested profile rendered as a single band.

## Host And Tools

- Host: macOS 26.5.1 (25F80), arm64.
- Rust: `rustc 1.95.0-nightly (842bd5be2 2026-01-29)`.
- Build: `cargo build --release -p ferrugo --no-default-features`.
- Profiler: macOS `sample`, 2-second captures against release
  `benchmark-repeat-native` processes.
- Timing tools: `benchmark-native`, `benchmark-repeat-native`, and
  `trace-native`.
- Artifact directory: `target/issue100-profile-refresh/`.

The `--max-edge 1024` runs use the requested large-render setting, but these
synthetic fixture page boxes naturally render below 1024 pixels on their long
edge. The report records both the requested max edge and actual output size.

## Timing Snapshot

Single-run means from `benchmark-native`:

| Fixture group | Max edge | Actual size | Mean ms | Main trace phase |
| --- | ---: | ---: | ---: | --- |
| `vector-stress.pdf` | 160 | 160x120 | 1.868 | `raster_paths=1.788 ms` |
| `vector-stress.pdf` | 1024 | 160x120 | 1.926 | `raster_paths=1.782 ms` |
| `report/vector` | 160 | 4 fixtures | 0.653-3.491 | path rasterization dominates |
| `report/vector` | 1024 | 4 fixtures | 1.817-10.971 | path rasterization dominates |
| `slide-title-gradient.pdf` | 160 | 160x90 | 0.131 | shading plus text |
| `slide-title-gradient.pdf` | 1024 | 320x180 | 0.325 | shading plus text |
| `office-report-header-footer-link.pdf` | 160 | 160x100 | 0.601 | paths plus text |
| `office-report-header-footer-link.pdf` | 1024 | 480x300 | 0.547 | paths plus text |

Repeat means from `benchmark-repeat-native`:

| Fixture | Max edge | Actual size | First ms | Repeat mean ms | Repeat raster paths ms | Repeat text ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `vector-stress.pdf` | 160 | 160x120 | 2.017 | 1.699 | 1.605 | 0.000 |
| `vector-stress.pdf` | 1024 | 160x120 | 2.012 | 1.695 | 1.601 | 0.000 |
| `technical-hatch-clipping.pdf` | 160 | 160x117 | 3.371 | 3.268 | 3.166 | 0.003 |
| `technical-hatch-clipping.pdf` | 1024 | 300x220 | 10.637 | 10.690 | 10.561 | 0.005 |
| `slide-title-gradient.pdf` | 160 | 160x90 | 0.252 | 0.090 | 0.048 | 0.017 |
| `slide-title-gradient.pdf` | 1024 | 320x180 | 2.005 | 0.250 | 0.190 | 0.028 |
| `office-report-header-footer-link.pdf` | 160 | 160x100 | 0.888 | 0.538 | 0.364 | 0.063 |
| `office-report-header-footer-link.pdf` | 1024 | 480x300 | 0.692 | 0.470 | 0.265 | 0.087 |

`benchmark-repeat-native` currently reports first/repeat mean/min/max rather
than p95. This report does not make a p95 performance claim.

## CPU Profile Findings

`vector-stress.pdf` at both max edges now samples primarily under
`stroke_path` routed into `fill_path` and
`FillScanlineCellAccumulator::fill_run`. The trace shows `66`
`outline_fill_calls`, `2` coverage-span fill calls, and `1,954`
coverage-cell accumulator pixels. The old row-bucket and span-covered stroke
routes report zero sample points in the current trace.

The broader `report/vector` sample is still path-raster dominated. At
`--max-edge 1024`, `technical-hatch-clipping.pdf` is the slowest member:
`10.690 ms` repeat mean with `10.561 ms` in `raster_paths`. The sample call
tree is led by `stroke_path`, `rasterize_path_item`, and `fill_path`, not by
the retired row-bucket/span-covered function names.

`slide-title-gradient.pdf` is dominated by `rasterize_shading_item`, with
secondary `draw_text_run` and text-rectangle fill work. That keeps SIMD row
blitters relevant, but the measured target is shading/text-row throughput, not
generic path rasterization.

`office-report-header-footer-link.pdf` shows mixed path and text work. At
`--max-edge 1024`, the sample is led by `stroke_path`, `draw_text_run`,
`fill_device_rect`, and a small `draw_image` tail. The trace recorded `142`
glyph bitmap cache hits and `81` misses, so #111 remains real but should be
measured against this fixture before changing cache budgets.

Every requested trace reported `bands=1`. The largest actual output in this
set was `480x300`, below the default banding threshold, so this pass does not
support making parallel banding a default for these families.

## Artifacts

- `target/issue100-profile-refresh/bench-vector-stress-160.json`
- `target/issue100-profile-refresh/bench-vector-stress-1024.json`
- `target/issue100-profile-refresh/bench-report-vector-160.json`
- `target/issue100-profile-refresh/bench-report-vector-1024.json`
- `target/issue100-profile-refresh/bench-presentation-160.json`
- `target/issue100-profile-refresh/bench-presentation-1024.json`
- `target/issue100-profile-refresh/bench-office-text-160.json`
- `target/issue100-profile-refresh/bench-office-text-1024.json`
- `target/issue100-profile-refresh/repeat-*.json`
- `target/issue100-profile-refresh/trace-*.json`
- `target/issue100-profile-refresh/sample-*.txt`

## Commands

Representative commands:

```sh
cargo build --release -p ferrugo --no-default-features
target/release/ferrugo benchmark-native fixtures/generated/vector-stress.pdf --max-edge 160 --iterations 20 --max-ms 1000000 --max-output-bytes 8388608 --output target/issue100-profile-refresh/bench-vector-stress-160.json
target/release/ferrugo benchmark-native fixtures/generated --manifest fixtures/performance-matrix-manifest.tsv --include-family report/vector --max-edge 1024 --iterations 10 --max-ms 1000000 --max-output-bytes 8388608 --output target/issue100-profile-refresh/bench-report-vector-1024.json
target/release/ferrugo trace-native fixtures/generated/office-report-header-footer-link.pdf --max-edge 1024 --max-events 1 --output target/issue100-profile-refresh/trace-office-text-1024.json
target/release/ferrugo benchmark-repeat-native fixtures/generated/slide-title-gradient.pdf --max-edge 1024 --repetitions 10000 --max-first-ms 1000000 --max-repeat-mean-ms 1000000 --output target/issue100-profile-refresh/repeat-presentation-1024.json
sample <pid> 2 -file target/issue100-profile-refresh/sample-presentation-1024.txt
```
