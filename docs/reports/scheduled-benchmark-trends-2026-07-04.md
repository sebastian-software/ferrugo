# Scheduled Benchmark Trends 2026-07-04

Issue: #115

## Decision

Ferrugo now has a weekly and manually dispatched scheduled benchmark workflow:

- `.github/workflows/scheduled-benchmarks.yml`
- `scripts/generate_scheduled_benchmark_artifacts.sh`

The workflow runs on `ubuntu-24.04`, installs Poppler, Ghostscript, and MuPDF,
and uploads benchmark, golden-image, visual-diff, and trend artifacts. These
artifacts are for shared-runner variance and regression visibility. They are
not claim-ready performance evidence by themselves.

## Artifacts

Each run uploads:

- `performance-matrix.json` and `performance-matrix.md`
- `real-world-performance-matrix.json` and `real-world-performance-matrix.md`
- `native-golden-comparison.json`
- `poppler-visual-diff.json`
- `benchmark-trend.jsonl`
- `benchmark-trend-summary.md`
- rendered matrix artifacts under the generated and real-world artifact
  directories

## Trend Rule

`benchmark-trend.jsonl` contains one JSON object per run. The scheduled script
can append an optional prior `TREND_HISTORY` file before writing the new series.
After at least five samples, `benchmark-trend-summary.md` reports the mean,
sample standard deviation, and coefficient of variation for the generated
`small-text` native hot-render p95 record.

The current threshold is CoV `0.20`. A scheduled run fails only when at least
five samples are available and that threshold is exceeded.

## Review Policy

- Shared-runner trends can surface variance and regressions.
- Promoted README or release claims still need the performance-claims checklist
  and may use dedicated hardware evidence.
- Missing PDFium remains acceptable; installed Poppler, Ghostscript, and MuPDF
  versions are recorded in matrix JSON through the oracle version metadata.
