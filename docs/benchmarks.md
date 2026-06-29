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

The best hard comparison currently available is the archived 0078
Rust-native/PDFium smoke run. It used the same generated fixture corpus,
`max_edge=160`, one iteration, and the shared thumbnail facade.

| Family | Ferrugo mean ms | PDFium mean ms | Read |
| --- | ---: | ---: | --- |
| `browser-print` | 38.607 | 0.665 | PDFium much faster |
| `form` | 19.343 | 7.912 | PDFium faster |
| `mixed-layout` | 17.910 | 2.338 | PDFium much faster |
| `office-export` | 15.449 | 30.339 | Ferrugo faster on this early slice |
| `presentation` | 15.452 | 0.382 | PDFium much faster |
| `report` | 267.832 | 0.581 | PDFium much faster; Ferrugo had a vector-stress tail |
| `scan` | 1.293 | 1.166 | roughly comparable |

The same run reported 50/52 Ferrugo-native renders, 1 typed fallback, and 1
encrypted error. PDFium rendered 51/52 with the same encrypted error. This is
why the project should not claim broad renderer performance parity yet.

Memory comparison is less complete, but now has a dedicated path. The Phase 0
PDFium release-CLI smoke measured roughly 24 MiB max RSS for `text-page.pdf` at
256-1024 max edge with 0.03-0.04s wall time. Ferrugo's native gates currently
enforce deterministic pixel, decoded-image, display-list, font, transparency,
cache, and output-byte budgets. The `benchmark-matrix` cold-process mode also
captures process peak RSS when `/usr/bin/time -l` is available, while hot-render
mode records start/end RSS samples for in-process backends.

Poppler is now included in the same cold-process matrix through `pdftoppm`.
MuPDF remains v2 backlog because setup, licensing, and tooling would slow the
first repeatable benchmark slice. A fair MuPDF claim still needs the same
first-page latency, output-size, and RSS fields across the same fixture
families. Public speed or memory copy must follow the
[performance claims policy](policies/performance-claims.md).

## Performance Matrix

Use `benchmark-matrix` for report-first performance work. It emits one JSON
schema for Ferrugo native, PDFium, and Poppler, grouped by an explicit manifest.
The default matrix covers both modes:

- `cold-process`: starts a CLI/tool process per fixture and records wall time,
  exit status, output bytes, output dimensions, and peak RSS when available.
- `hot-render`: runs in-process repetitions with warmup for Ferrugo native and
  PDFium, then reports mean, p50, p95, and max. Poppler is recorded as
  `not-applicable` in this mode because it is intentionally measured as an
  external tool.

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

Or call the CLI directly:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --max-edge 160 \
  --iterations 3 \
  --warmup 1 \
  --timeout 30 \
  --output target/performance-matrix.json \
  --report target/performance-matrix.md
```

If `FERRUGO_PDFIUM_LIBRARY` is set, the helper script enables the `pdfium`
feature. If PDFium or Poppler are missing, the matrix records `missing-tool`
rows instead of failing the run.

The Markdown report lists:

- top 25 slowest Ferrugo fixtures;
- top 25 largest cold-process gaps against the fastest reference renderer;
- top memory high-water records;
- family-level Ferrugo/PDFium and Ferrugo/Poppler ratios with p95/error counts.

This matrix is intentionally not a hard CI budget yet. First collect stable
artifacts, profile the top 5 Ferrugo fixtures with `sample`, Instruments, or
Samply on release builds, and only then open optimization PRs with before/after
evidence.

## Commands

Run the Rust-native benchmark against the generated fixture corpus:

```sh
cargo run -p ferrugo-cli -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-native-smoke.json
```

Run the PDFium baseline with the same budgets:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli --features pdfium -- benchmark-pdfium fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-pdfium-smoke.json
```

For a deeper local pass, increase both raster size and iterations:

```sh
cargo run -p ferrugo-cli -- benchmark-native fixtures/generated \
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
checks. It builds `ferrugo-cli` with the Cargo `serverless` profile, verifies
the CLI package file list does not contain PDFium/native runtime assets, then
measures:

- binary size from `target/serverless/ferrugo-cli`;
- process startup by invoking `ferrugo-cli --help`;
- first-render latency by invoking a new `render-native` process per sample.

The default fixture is `fixtures/generated/text-page.pdf` at `--max-edge 160`.
Override budgets with `FERRUGO_SERVERLESS_MAX_BINARY_BYTES`,
`FERRUGO_SERVERLESS_MAX_STARTUP_P95_MS`,
`FERRUGO_SERVERLESS_MAX_FIRST_RENDER_P95_MS`, and
`FERRUGO_SERVERLESS_MAX_RENDER_OUTPUT_BYTES`.
