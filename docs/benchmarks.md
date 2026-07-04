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
builds, and both `cold-process` and `hot-render` modes.

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
cold-process mode captures process peak RSS through `/usr/bin/time -l` when
available, while hot-render mode records sampled process RSS for in-process
backends. Ferrugo's native gates also enforce deterministic pixel,
decoded-image, display-list, font, transparency, cache, and output-byte budgets.

Poppler is now included in the same cold-process matrix through `pdftoppm`.
Ghostscript is also available as an external cold-process oracle through `gs`
or `FERRUGO_GHOSTSCRIPT`, with missing tools recorded as matrix data. MuPDF
remains v2 backlog because setup, licensing, and tooling would slow the first
repeatable benchmark slice. A fair MuPDF claim still needs the same first-page
latency, output-size, and RSS fields across the same fixture families. Public
speed or memory copy must follow the
[performance claims policy](policies/performance-claims.md).

## Performance Matrix

Use `benchmark-matrix` for report-first performance work. It emits one JSON
schema for Ferrugo native, PDFium, Poppler, and Ghostscript, grouped by an
explicit manifest. The default matrix covers both modes:

- `cold-process`: starts a CLI/tool process per fixture and records wall time,
  exit status, output bytes, output dimensions, and peak RSS when available.
- `hot-render`: runs in-process repetitions with warmup for Ferrugo native and
  PDFium, then reports sample count, mean, sample standard deviation,
  coefficient of variation (CoV), interpolated p50/p95, and max. Poppler and
  Ghostscript are recorded as `not-applicable` in this mode because they are
  intentionally measured as external tools.

The focused starter manifest is `fixtures/performance-matrix-manifest.tsv`.
It maps the initial families to real generated fixtures:

- `small-text`
- `office-export`
- `scan`
- `browser-print`
- `form`
- `presentation`
- `report/vector`
- `mixed-layout`

Run the repeatable matrix:

```sh
bash scripts/generate_performance_matrix.sh
```

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

If `FERRUGO_PDFIUM_LIBRARY` is set, the helper script enables the `pdfium`
feature. Set `FERRUGO_GHOSTSCRIPT=/path/to/gs` when Ghostscript is not on
`PATH`. If PDFium, Poppler, or Ghostscript are missing, the matrix records
`missing-tool` rows instead of failing the run.

The Markdown report lists:

- top 25 slowest Ferrugo fixtures;
- top 25 largest cold-process gaps against the fastest reference renderer;
- top memory high-water records;
- family-level Ferrugo/PDFium, Ferrugo/Poppler, and Ferrugo/Ghostscript ratios
  with p95/error counts.

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

## Benchmark Suite Tiers

Ferrugo uses named benchmark tiers instead of one oversized performance job.
Each tier has a different review purpose and failure policy.

| Tier | Command | Required tools | Failure policy | Artifacts |
| --- | --- | --- | --- | --- |
| Smoke | `bash scripts/check_benchmark_suite.sh` | Rust toolchain only | Fails on missing schema/platform/timing fields, native errors, native fallbacks, or missing tools. Does not fail on broad timing comparisons. | `target/benchmark-suite/performance-matrix-smoke.json`, `target/benchmark-suite/performance-matrix-smoke.md`, `target/benchmark-suite/benchmark-suite-summary.txt` |
| Release candidate | `bash scripts/check_native_only_release.sh` | Rust toolchain only | Includes the smoke tier and the native-only release gates. Stable budget failures are allowed only for bounded native release checks, not PDFium/Poppler availability or noisy cross-renderer ratios. | Native release artifacts plus the smoke tier artifacts. |
| Local deep | `bash scripts/generate_performance_matrix.sh` | Rust toolchain; optional PDFium and Poppler | Does not block release by itself. Use it to collect repeated artifacts, inspect `timing_reliability`, and guide optimization issues. | `target/performance-matrix.json`, `target/performance-matrix.md`, `target/performance-matrix-artifacts/` |
| Maintainer oracle comparison | `FERRUGO_PDFIUM_LIBRARY=/path/to/libpdfium.dylib bash scripts/generate_performance_matrix.sh` plus Poppler when available | PDFium and/or Poppler | Missing reference tools must be recorded as `missing-tool`, not hidden. Public claims need two stable runs and the performance-claims checklist. | Dated JSON/Markdown reports under `target/` or `docs/reports/` when promoted. |

The smoke and release-candidate tiers run from a clean checkout with only
committed generated fixtures. They do not require private corpus files, PDFium,
Poppler, or network access. The initial release smoke uses the `small-text`
family in native `hot-render` mode at `max_edge=120`; that subset is deliberately
small enough to be stable while still proving the durable `benchmark-matrix`
JSON, Markdown, platform, timing, sample-count, stddev, CoV, family, and record
fields.

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

Run the PDFium baseline with the same budgets:

```sh
FERRUGO_PDFIUM_LIBRARY=/path/to/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/path/to/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo --features pdfium -- benchmark-pdfium fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-pdfium-smoke.json
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

- `backend`: `rust-native` or `pdfium`.
- `platform`: target `os`, `arch`, `family`, `endian`, and
  `pointer_width_bits`.
- `config`: iteration count, render-time budget, and output-byte budget.
- `summary`: total fixture count, native render count, fallback count, error
  count, and budget-failure count.
- `families`: grouped totals and timing/output aggregates by manifest family.
- `fixtures`: per-file outcome and budget violations.

The field name `native_rendered` means "rendered by the selected benchmark
backend" in the generic report schema. For PDFium reports, it indicates PDFium
successes.

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

The legacy `benchmark-native` and `benchmark-pdfium` reports deliberately do
not report operating-system peak RSS. Memory expectations remain enforced
through deterministic renderer budgets documented in
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
