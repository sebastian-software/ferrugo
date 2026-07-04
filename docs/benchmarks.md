# Renderer Benchmarks

Status: accepted.
Date: 2026-06-24.

The benchmark harness measures whether `ferrugo` can do its main job quickly
and predictably: produce bounded preview images for common document families.
It uses the public thumbnail facade and emits JSON reports grouped by corpus
family, so timing, fallbacks, errors, and budget violations stay visible.

Reference-renderer benchmark commands exist for maintainers when a local
comparison library is available. They are not part of the normal runtime path.

## Current Local Snapshot

Latest local smoke run after the Ferrugo rename, on macOS/aarch64:

| Gate | Result |
| --- | ---: |
| Low-memory corpus | 5/5 native, 0 fallbacks, 0 errors, 0 budget failures |
| Low-memory common docs | 4.815 ms mean |
| Low-memory scan fixture | 41.876 ms mean |
| Low-memory vector-stress fixture | 139.301 ms mean |
| Server batch | 16/16 jobs native, 0 fallbacks, 0 errors, 0 budget failures |
| Server batch throughput | 38.025 jobs/sec |
| Server batch latency | 28.381 ms mean, 8.847 ms p50, 139.118 ms p95 |
| Server batch bounds | 2 workers, 51200 in-flight pixels, 78720 max output bytes |

Older release-readiness evidence also records a size-oriented serverless CLI
binary around 1.0 MB and first-render p95 below 6 ms for the small text fixture.
These numbers are useful for direction and regression checks, not as universal
hardware-independent guarantees.

## Comparison Against Existing Renderers

The current PDFium comparison reference is the 2026-07-03 two-run
performance-matrix refresh in
[`docs/reports/pdfium-comparison-refresh-2026-07-03.md`](reports/pdfium-comparison-refresh-2026-07-03.md).
It used the pinned PDFium revision
`573758fe2dd928279cd52b5a4bc955a6938aab39`, the generated
`fixtures/performance-matrix-manifest.tsv` corpus, `max_edge=160`, release
builds, and both `cold-process` and `hot-render` modes. The PDFium matrix path
has since moved to an external process oracle; use this report as historical
same-corpus evidence, not as the current command recipe.

Both runs reported 44/44 rendered records with no fallbacks, missing tools,
not-applicable rows, errors, or timing reliability caveats. RSS was available
in both runs.

Current policy-compliant read:

- Ferrugo native hot-render p95 was below PDFium in every measured family.
- Ferrugo native peak RSS was below PDFium in every measured family and mode.
- Ferrugo native cold-process wall time was below PDFium in all families except
  the `form` run-2 tie/noise case (`1.007x` Ferrugo/PDFium after run 1 measured
  `0.536x`).
- These are family-, mode-, metric-, host-, corpus-, and revision-scoped
  observations, not a broad renderer-parity claim.

The archived 0078 Rust-native/PDFium smoke run from 2026-06-24 is superseded
for the focused performance-matrix corpus. Keep it only as historical context
for early Phase 0 work; do not use its 8x-460x PDFium gap as the current
reference.

Memory comparison now has a dedicated path. The `benchmark-matrix`
cold-process mode captures process peak RSS through BSD `/usr/bin/time -l` or
GNU `/usr/bin/time -v` when available. Hot-render mode records Linux
`/proc/self/status` `VmHWM` high-water RSS when available, otherwise it falls
back to sampled process RSS. Ferrugo's native gates also enforce deterministic
pixel, decoded-image, display-list, font, transparency, cache, and output-byte
budgets.

PDFium, Poppler, Ghostscript, and MuPDF are external cold-process oracles.
PDFium is configured through `--pdfium PATH` or `FERRUGO_PDFIUM_RENDERER`;
Poppler uses `pdftoppm`; Ghostscript uses `gs` or `FERRUGO_GHOSTSCRIPT`; MuPDF
uses `mutool` or `FERRUGO_MUTOOL`. Missing tools are recorded as matrix data.
The matrix also records the pinned oracle version lockfile status from
`fixtures/reference-renderers.lock.tsv`; version drift appears as a reliability
caveat and in JSON config metadata. Public speed or memory copy must follow the
[performance claims policy](policies/performance-claims.md).

## Performance Matrix

Use `benchmark-matrix` for report-first performance work. It emits one JSON
schema for Ferrugo native, PDFium, Poppler, Ghostscript, and MuPDF, grouped by
an explicit manifest. The default matrix covers both modes:

- `cold-process`: starts a CLI/tool process per fixture and records wall time,
  startup-adjusted wall time, exit status, output bytes, output dimensions, and
  peak RSS when available. Successful cold-process rows fail as
  `output-dimension-mismatch` if the renderer output does not match the native
  target dimensions, or the requested max edge when no native target exists.
- `hot-render`: runs in-process repetitions with warmup for Ferrugo native,
  then reports sample count, mean, sample standard deviation, coefficient of
  variation (CoV), interpolated p50/p95, and max. PDFium, Poppler,
  Ghostscript, and MuPDF are recorded as `not-applicable` in this mode because
  they are intentionally measured as external tools.

The focused starter manifest is `fixtures/performance-matrix-manifest.tsv`.
It maps the initial generated families plus the #97 real-world seed families:

- `small-text`
- `office-export`
- `scan`
- `browser-print`
- `form`
- `presentation`
- `report/vector`
- `mixed-layout`
- `real-office-export`
- `real-browser-print`
- `real-scan`
- `real-report`

Run the repeatable matrix:

```sh
bash scripts/generate_performance_matrix.sh
```

Run the committed real-world seed tier at larger preview scale:

```sh
INPUT=fixtures/real-world MAX_EDGE=1024 ITERATIONS=5 WARMUP=1 \
  OUTPUT=target/real-world-performance-matrix.json \
  REPORT=target/real-world-performance-matrix.md \
  ARTIFACT_DIR=target/real-world-performance-matrix-artifacts \
  bash scripts/generate_performance_matrix.sh
```

Those real-world rows include multi-page documents and high-DPI benchmark tags
so the same matrix schema can show public form, browser-print, scan, and report
families separately from synthetic generated fixtures. Keep the smoke gate on
the generated `small-text` subset; the real-world tier is evidence for review
and trend artifacts until #115/#114 promote scheduled benchmark publication.

Run the budget-free native smoke gate before wiring a focused subset into CI:

```sh
bash scripts/check_performance_matrix_smoke.sh
```

Run the provider-neutral multi-oracle smoke when changing external oracle
plumbing:

```sh
bash scripts/check_multi_oracle_smoke.sh
```

Or call the CLI directly:

```sh
cargo run -p ferrugo --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge 160 \
  --iterations 20 \
  --warmup 3 \
  --max-cov 0.15 \
  --timeout 30 \
  --output target/performance-matrix.json \
  --report target/performance-matrix.md
```

Set `FERRUGO_PDFIUM_RENDERER=/path/to/pdfium-renderer` or pass
`--pdfium /path/to/pdfium-renderer` for the external PDFium oracle. Set
`FERRUGO_GHOSTSCRIPT=/path/to/gs` when Ghostscript is not on `PATH`; set
`FERRUGO_MUTOOL=/path/to/mutool` for MuPDF. If PDFium, Poppler, Ghostscript, or
MuPDF are missing, the matrix records `missing-tool` rows instead of failing the
run.

The Markdown report lists:

- top 25 slowest Ferrugo fixtures;
- top 25 largest cold-process gaps against the fastest reference renderer;
- top memory high-water records;
- family-level Ferrugo/PDFium, Ferrugo/Poppler, Ferrugo/Ghostscript, and
  Ferrugo/MuPDF ratios with p95/error counts.

This matrix is intentionally not a broad cross-renderer CI budget yet. First
collect stable artifacts, profile the top 5 Ferrugo fixtures with `sample`,
Instruments, or Samply on release builds, and only then open optimization PRs
with before/after evidence.

For benchmark-matrix evidence, "stable" means each hot-render record used at
least 20 measured samples after a separate warmup phase, includes sample
standard deviation and CoV, and stays at or below the configured CoV threshold.
Public claims use `--max-cov 0.15` unless the report explicitly documents a
stricter threshold. Smoke gates may use a looser threshold to validate schema
and harness plumbing without turning local scheduling noise into product copy.

## Promoted README Results

README performance results are generated from a promoted benchmark-matrix JSON
series under `docs/benchmarks/promoted/`. Regenerate the README block and the
standardized report after promoting a new artifact:

```sh
node scripts/generate_readme_benchmark_results.mjs --write
```

Validate that the README and report still match the promoted artifact:

```sh
bash scripts/check_readme_benchmark_results.sh
```

The generated report records host/platform metadata, fixture set, renderer
version lock status, reliability caveats, per-family Ferrugo hot p95, cold
process timings for each external oracle, and RSS where available. Keep the
README wording scoped to that artifact; stronger public claims still require the
performance-claims checklist.

## Benchmark Suite Tiers

Ferrugo uses named benchmark tiers instead of one oversized performance job.
Each tier has a different review purpose and failure policy.

| Tier | Command | Required tools | Failure policy | Artifacts |
| --- | --- | --- | --- | --- |
| Smoke | `bash scripts/check_benchmark_suite.sh` | Rust toolchain only | Fails on missing schema/platform/timing fields, native errors, native fallbacks, or missing tools. Does not fail on broad timing comparisons. | `target/benchmark-suite/performance-matrix-smoke.json`, `target/benchmark-suite/performance-matrix-smoke.md`, `target/benchmark-suite/benchmark-suite-summary.txt` |
| Release candidate | `bash scripts/check_native_only_release.sh` | Rust toolchain only | Includes the smoke tier and the native-only release gates. Stable budget failures are allowed only for bounded native release checks, not PDFium/Poppler availability or noisy cross-renderer ratios. | Native release artifacts plus the smoke tier artifacts. |
| Local deep | `bash scripts/generate_performance_matrix.sh`; use `INPUT=fixtures/real-world MAX_EDGE=1024` for the committed real-world seed tier | Rust toolchain; optional PDFium, Poppler, and Ghostscript | Does not block release by itself. Use it to collect repeated artifacts, inspect `timing_reliability`, and guide optimization issues. | `target/performance-matrix.json`, `target/performance-matrix.md`, `target/performance-matrix-artifacts/` |
| Scheduled trends | `.github/workflows/scheduled-benchmarks.yml` or `bash scripts/generate_scheduled_benchmark_artifacts.sh` | Rust toolchain; Poppler, Ghostscript, and MuPDF on the runner | Runs weekly and on demand. Fails only when the generated trend summary has at least five samples and the small-text native hot p95 CoV exceeds the configured trend threshold. Shared-runner results characterize variance, not claim-ready hardware performance. | `target/scheduled-benchmarks/performance-matrix.json`, `real-world-performance-matrix.json`, `native-golden-comparison.json`, `poppler-visual-diff.json`, `benchmark-trend.jsonl`, `benchmark-trend-summary.md` |
| Maintainer oracle comparison | `FERRUGO_PDFIUM_RENDERER=/path/to/pdfium-renderer bash scripts/generate_performance_matrix.sh` plus Poppler when available | PDFium and/or Poppler | Missing reference tools must be recorded as `missing-tool`, not hidden. Public claims need two stable runs and the performance-claims checklist. | Dated JSON/Markdown reports under `target/` or `docs/reports/` when promoted. |

The smoke and release-candidate tiers run from a clean checkout with only
committed generated fixtures. They do not require private corpus files, PDFium,
Poppler, or network access. The initial release smoke uses the `small-text`
family in native `hot-render` mode at `max_edge=120`; that subset is deliberately
small enough to be stable while still proving the durable `benchmark-matrix`
JSON, Markdown, platform, timing, sample-count, stddev, CoV, family, and record
fields.

The scheduled tier installs Poppler, Ghostscript, and MuPDF on `ubuntu-24.04`,
runs generated and real-world matrices, runs the native golden comparison, and
runs a Poppler visual-diff suite over the cross-producer fusion manifest. It
uploads all JSON/Markdown artifacts instead of committing generated reports.
The trend JSONL is built from the current run plus an optional prior
`TREND_HISTORY` file; after five samples it reports mean, sample standard
deviation, and CoV for the `small-text` native hot p95 record and can fail the
job when the CoV exceeds the trend threshold. Use these artifacts to decide
whether a dedicated-hardware claim run is warranted.

## Release Candidate Artifact Policy

Release-candidate benchmark artifacts are evidence, not marketing copy. Keep
them under `target/benchmark-suite/` for local gates and promote a dated report
under `docs/reports/` only when it supports a specific release or performance
claim.

Required fields for release-candidate benchmark evidence:

- `schema_version` and `report_kind` in JSON;
- `platform.os`, `platform.arch`, and compiler/runtime metadata when available;
- `config` with `input`, `manifest`, `include_families`, `max_edge`,
  `iterations`, `warmup`, `max_cov`, backend list, mode list, and native
  profile;
- `timing_reliability` with caveats reviewed before public copy changes;
- `summary`, `families`, and per-record `timing`, `output`, `memory`, and
  `status` fields;
- per-record `timing.sample_count`, `timing.stddev_ms`, `timing.cov`,
  `timing.cov_threshold`, and `timing.cov_exceeded`;
- Markdown report generated from the same JSON artifact.

Retention rules:

- `target/benchmark-suite/*` is disposable local gate output.
- PRs should list the commands and artifact paths they produced.
- Dated reports in `docs/reports/` should include host details, command lines,
  artifact names, and how missing PDFium/Poppler tools were handled.
- Public README or release copy must cite promoted evidence and pass
  `bash scripts/check_performance_claims.sh`.

## Commands

Run the Rust-native benchmark against the generated fixture corpus:

```sh
cargo run -p ferrugo -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-native-smoke.json
```

For a deeper local pass, increase both raster size and iterations:

```sh
cargo run -p ferrugo -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 320 \
  --iterations 3 \
  --max-ms 10000 \
  --max-output-bytes 4194304 \
  --output target/benchmark-native-deep.json
```

## Legacy Report Schema

Each report includes:

- `backend`: `rust-native`.
- `platform`: target `os`, `arch`, `family`, `endian`, and
  `pointer_width_bits`.
- `config`: iteration count, render-time budget, and output-byte budget.
- `summary`: total fixture count, native render count, fallback count, error
  count, and budget-failure count.
- `families`: grouped totals and timing/output aggregates by manifest family.
- `fixtures`: per-file outcome and budget violations.

The field name `native_rendered` means "rendered by the selected benchmark
backend" in the generic report schema. Historical PDFium reports used the same
field name for PDFium successes, but new PDFium comparison work should use
`benchmark-matrix`.

## Budget Policy

Smoke budgets should be stable enough for local regression checks:

- `--max-edge 160`
- `--iterations 1`
- `--max-ms 1000`
- `--max-output-bytes 1048576`

Deep local runs should use larger rasters or more iterations, but they should
not become release-blocking until variance is characterized across machines.
Use `--fail-on-budget` only when the selected corpus and machine budget are
known to be stable.

Budget violations are typed:

- `render_time`: mean fixture render time exceeded `--max-ms`.
- `output_bytes`: output RGBA bytes exceeded `--max-output-bytes`.
- `native_fallback`: Rust-native reported an unsupported feature that requires
  PDFium fallback.
- `render_error`: the selected backend returned a non-fallback render error.

The legacy `benchmark-native` report deliberately does not report
operating-system peak RSS. Memory expectations remain enforced through
deterministic renderer budgets documented in
`docs/policies/renderer-memory-budgets.md`; legacy benchmark output bytes are
only a lightweight allocation proxy. Use `benchmark-matrix` for cross-renderer
RSS fields.

## Serverless Cold Start

Use `scripts/measure_serverless_profile.sh` for short-lived native-only worker
checks. It builds `ferrugo` with the Cargo `serverless` profile, verifies
the CLI package file list does not contain PDFium/native runtime assets, then
measures:

- binary size from `target/serverless/ferrugo`;
- process startup by invoking `ferrugo --help`;
- first-render latency by invoking a new `render-native` process per sample.

The default fixture is `fixtures/generated/text-page.pdf` at `--max-edge 160`.
Override budgets with `FERRUGO_SERVERLESS_MAX_BINARY_BYTES`,
`FERRUGO_SERVERLESS_MAX_STARTUP_P95_MS`,
`FERRUGO_SERVERLESS_MAX_FIRST_RENDER_P95_MS`, and
`FERRUGO_SERVERLESS_MAX_RENDER_OUTPUT_BYTES`.
