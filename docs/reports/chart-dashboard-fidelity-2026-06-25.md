# Chart Dashboard Fidelity 2026-06-25

Milestone: 0125.

## Decision

Chart, dashboard, and map-style thumbnails now have a focused native gate. The
native renderer renders all eight chart-dashboard manifest rows without PDFium
fallback, errors, or benchmark budget failures.

PDFium remains a maintainer-only visual oracle. Current strict visual-diff
thresholds classify six rows as fidelity blockers and two rows as exact
matches, with no native or PDFium render errors.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `chart-combo-legend.pdf` | chart | bar chart, trend line, axis, legend, small labels |
| `dashboard-kpi-panels.pdf` | dashboard | KPI panels, translucent overlays, sparkline, table lines |
| `map-marker-clusters.pdf` | map | map regions, dashed routes, clustered markers, labels |
| `dashboard-heatmap-overlay.pdf` | dashboard | heatmap tiles, translucent overlay, grid, labels |

`fixtures/chart-dashboard-manifest.tsv` combines these with existing chart
slide, clipping, vector-stress, and tiling-pattern baselines.

## Native Gate Evidence

Artifact: `target/chart-0125-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `chart` | 2 | 2 | 0 | 0 |
| `dashboard` | 2 | 2 | 0 | 0 |
| `map` | 2 | 2 | 0 | 0 |
| `marker-heavy` | 2 | 2 | 0 | 0 |
| **Total** | **8** | **8** | **0** | **0** |

The native regression test also checks visible non-background pixel counts so
markers, labels, legends, and panels cannot silently collapse to empty output.

## Benchmark Evidence

Artifact: `target/chart-0125-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `chart` | 2 | 2 | 30.333 | 39.484 | 0 |
| `dashboard` | 2 | 2 | 31.968 | 33.887 | 0 |
| `map` | 2 | 2 | 76.830 | 105.950 | 0 |
| `marker-heavy` | 2 | 2 | 112.674 | 192.562 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/chart-0125-visual-diff.json`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `chart` | 2 | 0 | 0 | 2 | 0 | 0 |
| `dashboard` | 2 | 0 | 0 | 2 | 0 | 0 |
| `map` | 2 | 1 | 0 | 1 | 0 | 0 |
| `marker-heavy` | 2 | 1 | 0 | 1 | 0 | 0 |
| **Total** | **8** | **2** | **0** | **6** | **0** | **0** |

The remaining blockers are visual-fidelity work around small-label metrics,
alpha compositing differences, marker edge placement, and repeated vector
stroke antialiasing, not native coverage failures.

## Follow-Up Backlog

- Improve small chart-label and legend text positioning against PDFium.
- Tighten alpha compositing for dashboard overlays.
- Profile marker-heavy vector workloads as part of future hot-path work.
- Add producer-derived dashboard and map reductions once private examples can
  be sanitized.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/chart-dashboard-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo test -p pdfrust-native chart_dashboard -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/chart-dashboard-manifest.tsv --include-family chart --include-family dashboard --include-family map --include-family marker-heavy --fail-on-fallback --max-edge 160 --output target/chart-0125-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/chart-dashboard-manifest.tsv --include-family chart --include-family dashboard --include-family map --include-family marker-heavy --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/chart-0125-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/chart-dashboard-manifest.tsv --include-family chart --include-family dashboard --include-family map --include-family marker-heavy --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/chart-0125-visual-diff.json
```
