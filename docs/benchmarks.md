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

## Report Schema

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

The harness deliberately does not report operating-system peak RSS. Memory
expectations remain enforced through deterministic renderer budgets documented
in `docs/policies/renderer-memory-budgets.md`; benchmark output bytes are only
a lightweight allocation proxy.

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
