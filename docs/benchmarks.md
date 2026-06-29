# Renderer Benchmarks

Status: accepted.
Date: 2026-06-24.

The renderer benchmark harness measures fixture-level render behavior for both
the Rust-native backend and PDFium when a local PDFium library is available.
Benchmarks use the existing thumbnail facade and emit JSON reports that group
results by corpus family.

## Commands

Run the Rust-native benchmark against the generated fixture corpus:

```sh
cargo run -p pdfrust-cli -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-native-smoke.json
```

Run the PDFium baseline with the same budgets:

```sh
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib \
cargo run -p pdfrust-cli --features pdfium -- benchmark-pdfium fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/benchmark-pdfium-smoke.json
```

For a deeper local pass, increase both raster size and iterations:

```sh
cargo run -p pdfrust-cli -- benchmark-native fixtures/generated \
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
checks. It builds `pdfrust-cli` with the Cargo `serverless` profile, verifies
the CLI package file list does not contain PDFium/native runtime assets, then
measures:

- binary size from `target/serverless/pdfrust-cli`;
- process startup by invoking `pdfrust-cli --help`;
- first-render latency by invoking a new `render-native` process per sample.

The default fixture is `fixtures/generated/text-page.pdf` at `--max-edge 160`.
Override budgets with `PDFRUST_SERVERLESS_MAX_BINARY_BYTES`,
`PDFRUST_SERVERLESS_MAX_STARTUP_P95_MS`,
`PDFRUST_SERVERLESS_MAX_FIRST_RENDER_P95_MS`, and
`PDFRUST_SERVERLESS_MAX_RENDER_OUTPUT_BYTES`.
