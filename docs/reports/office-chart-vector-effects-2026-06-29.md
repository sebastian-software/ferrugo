# Office Chart SmartArt And Vector Effect Fidelity

Date: 2026-06-29
Milestone: 0204

## Decision

The office chart and vector-effect slice is native-runtime ready, but not
visual-parity complete. Focused support, operator coverage, and benchmark gates
pass with zero native fallbacks, native errors, or budget failures. Independent
Poppler review still exposes two high-signal visual drifts, so this milestone
adds the combined gate and records the follow-up renderer work instead of
reintroducing a PDFium runtime path.

## Corpus

0204 adds `fixtures/office-chart-vector-effects-manifest.tsv`.

The manifest combines typical office chart and SmartArt-style reductions:

- chart legends and small labels
- spreadsheet chart overlays
- rotated slide callouts
- gradient-heavy title slides
- grouped office vector shapes
- nested vector clipping
- clipped transparency groups
- repeated vector effects

The manifest is a renderer fidelity gate. It does not attempt to reconstruct
editable chart or SmartArt semantics.

## Native Support Gate

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 10 | 10 | 0 | 0 |

## Benchmark And Memory Budget

| Total | Native rendered | Fallbacks | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 10 | 0 | 0 | 0 |

The benchmark gate used `--max-edge 160`, two iterations, `--max-ms 1000`, and
`--max-output-bytes 1048576`. This keeps nested vector and Form XObject style
workloads bounded for the server-side renderer path.

## Operator Coverage

| Total fixtures | Scanned | Errors | Total operators | Implemented | Partial | Unsupported | Ignored |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 10 | 0 | 964 | 955 | 9 | 0 | 0 |

Partial operator buckets:

| Operator | Count | Bucket |
| --- | ---: | --- |
| `W` | 5 | `graphics.stroke-clip` |
| `gs` | 3 | `graphics.transparency` |
| `sh` | 1 | `graphics.pattern-shading` |

These are tracked as partial because the native renderer supports the common
subset used by the gate, while full PDF conformance still has additional
stroke/clip, transparency, and shading semantics.

## Poppler Visual Review

Poppler is used here only as an independent review oracle. It is not part of
the supported runtime path.

| Total | Accepted drift | Blockers | Native errors | Reference errors |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 4 | 2 | 0 | 4 |

Visual blockers:

| Fixture | Family | Subsystem | Main metrics |
| --- | --- | --- | --- |
| `slide-rotated-callout.pdf` | `slide-chart-callout` | `page-geometry` | MAE 2.730, p95 8, changed 0.839 |
| `slide-title-gradient.pdf` | `gradient-slide` | `vector-graphics` | MAE 7.213, p95 55, changed 0.503 |

Reference errors are Poppler timeouts in the local review run:

- `chart-combo-legend.pdf`
- `financial-chart-summary.pdf`
- `office-spreadsheet-chart-comments.pdf`
- `office-vector-clipped-transparency-group.pdf`

Those are not native renderer errors. They stay visible so future review runs
can separate reference-tool limits from native regressions.

## Follow-Ups

1. Tighten rotated text geometry and text metric placement for slide callouts.
2. Reduce axial shading drift on gradient-heavy slides without adding
   unbounded gradient caches.
3. Keep `W`, `gs`, and `sh` partial operator buckets visible until their
   remaining PDF semantics are covered by conformance tests.
4. Revisit Poppler timeout behavior for chart-heavy generated references before
   treating reference errors as release blockers.

## Validation

Commands run:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --fail-on-fallback --max-edge 160 --output target/office-chart-0204-support.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/office-chart-0204-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --output target/office-chart-0204-operators.json
cargo run -p ferrugo-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/office-chart-0204-poppler.json
cargo fmt --check
cargo test -p ferrugo-native native_backend_should_render_generated_office_vector_effect_fixtures -- --nocapture
cargo test -p ferrugo-native native_backend_should_render_generated_presentation_slide_fixtures -- --nocapture
cargo test -p ferrugo-native native_backend_should_render_generated_chart_dashboard_fixtures -- --nocapture
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
