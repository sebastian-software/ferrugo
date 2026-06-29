# 0204: Office Chart SmartArt And Vector Effect Fidelity

Status: done
Phase: 38
Size: medium
Depends on: 0203

## Goal

Improve Rust-native rendering for common office chart, diagram, SmartArt-style,
and vector-effect PDFs produced by presentation and document suites.

## Scope

- Add reduced fixtures for chart fills, shadows, clipped legends, connectors,
  grouped vector effects, and gradient-heavy diagram exports.
- Measure transparency, clipping, pattern, and text-overlay interactions inside
  nested form XObjects.
- Track chart-specific visual drift separately from generic vector stress.
- Prioritize implementation work that improves typical office documents without
  adding unbounded caches.

## Non-Goals

- Reconstruct editable chart or SmartArt semantics.
- Support every proprietary office effect exactly.
- Add PDFium runtime fallback for chart pages.

## Deliverables

- Office chart and vector-effect corpus.
- Fidelity report with reduced failure examples.
- Prioritized renderer fixes for high-impact effects.

## Acceptance Criteria

- Common chart and diagram exports render with stable fills, strokes, clipping,
  and labels.
- Unsupported effects produce typed diagnostics.
- Nested vector effects do not exceed memory or recursion budgets.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run office chart visual comparisons.
- Run form XObject recursion and memory-budget checks.
- Run operator coverage scan for chart fixtures.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `fixtures/office-chart-vector-effects-manifest.tsv` as the combined
  gate for chart legends, spreadsheet chart overlays, rotated slide callouts,
  gradient slides, grouped vector shapes, nested clips, clipped transparency
  groups, and repeated vector effects.
- Documented the manifest in `docs/corpus-taxonomy.md`.
- Recorded the 0204 fidelity baseline in
  `docs/reports/office-chart-vector-effects-2026-06-29.md`.
- Support and benchmark gates pass for all 10 fixtures with zero fallbacks,
  native errors, or budget failures.
- Operator coverage scanned 964 operators: 955 implemented, 9 partial, and 0
  unsupported.
- Poppler review found 4 accepted drifts, 2 native visual blockers, and 4
  Poppler reference timeouts. The remaining native blockers are
  `slide-rotated-callout.pdf` geometry drift and `slide-title-gradient.pdf`
  gradient/vector drift.

Validation run:

- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --fail-on-fallback --max-edge 160 --output target/office-chart-0204-support.json`
- `cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/office-chart-0204-benchmark.json`
- `cargo run -p pdfrust-cli --no-default-features -- operator-coverage fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --output target/office-chart-0204-operators.json`
- `cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/office-chart-vector-effects-manifest.tsv --include-family chart-legend --include-family chart-table-overlay --include-family slide-chart-callout --include-family gradient-slide --include-family grouped-vector --include-family nested-vector-clips --include-family repeated-vector-effects --max-edge 120 --max-mae 8 --max-p95 64 --max-changed-ratio 0.20 --output target/office-chart-0204-poppler.json`
- `cargo fmt --check`
- `cargo test -p pdfrust-native native_backend_should_render_generated_office_vector_effect_fixtures -- --nocapture`
- `cargo test -p pdfrust-native native_backend_should_render_generated_presentation_slide_fixtures -- --nocapture`
- `cargo test -p pdfrust-native native_backend_should_render_generated_chart_dashboard_fixtures -- --nocapture`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
