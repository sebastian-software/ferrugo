# Renderer Benchmark Suite 2026-06-24

This report records milestone 0078 coverage for repeatable renderer benchmark
budgets and native/PDFium baseline comparison.

## Implemented Slice

- Added `pdfrust-cli benchmark-native` for Rust-native corpus benchmark runs.
- Added `pdfrust-cli benchmark-pdfium` for PDFium baseline runs when
  `PDFRUST_PDFIUM_LIBRARY` points at a local dynamic library.
- Reused `ThumbnailBackend` so both benchmark commands exercise the same
  thumbnail facade and option surface.
- Added JSON report output grouped by `fixtures/corpus-manifest.tsv` families.
- Added fixture-level budget violations for render time, output bytes,
  Rust-native fallback, and backend render errors.
- Added CLI tests for default budget parsing and family/budget aggregation.

## Smoke Baselines

Commands:

```text
cargo run -p pdfrust-cli -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/benchmark-native-0078-smoke.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli -- benchmark-pdfium fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/benchmark-pdfium-0078-smoke.json
```

Native summary:

| Total | Rendered | Fallback | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 52 | 50 | 1 | 1 | 3 |

PDFium summary:

| Total | Rendered | Fallback | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 52 | 51 | 0 | 1 | 1 |

Native family timings:

| Family | Rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| browser-print | 4 | 38.607 | 79.101 | 0 |
| form | 6 | 19.343 | 36.846 | 0 |
| mixed-layout | 8 | 17.910 | 64.722 | 1 |
| office-export | 10 | 15.449 | 78.452 | 0 |
| presentation | 3 | 15.452 | 22.396 | 1 |
| report | 12 | 267.832 | 2872.257 | 1 |
| scan | 7 | 1.293 | 2.111 | 0 |

PDFium family timings:

| Family | Rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: |
| browser-print | 4 | 0.665 | 0.747 | 0 |
| form | 6 | 7.912 | 45.449 | 0 |
| mixed-layout | 8 | 2.338 | 15.816 | 1 |
| office-export | 10 | 30.339 | 158.700 | 0 |
| presentation | 4 | 0.382 | 0.487 | 0 |
| report | 12 | 0.581 | 0.996 | 0 |
| scan | 7 | 1.166 | 3.117 | 0 |

The expected smoke budget failures are:

- `encrypted-placeholder.pdf`: render error for both backends.
- `optional-content-ocmd.pdf`: Rust-native fallback requirement.
- `vector-stress.pdf`: Rust-native render-time budget violation at the smoke
  threshold.

## Deep Local Baseline

Command:

```text
cargo run -p pdfrust-cli -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 320 --iterations 3 --max-ms 10000 --max-output-bytes 4194304 --output target/benchmark-native-0078-deep.json
```

Summary:

| Total | Rendered | Fallback | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 52 | 50 | 1 | 1 | 2 |

The deep run keeps the same known encrypted error and OCMD fallback. With the
larger local budget, `vector-stress.pdf` no longer fails the render-time
budget.

## Validation

```text
cargo fmt --check
cargo test -p pdfrust-cli benchmark -- --nocapture
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

All commands completed successfully.

## Remaining Limits

- Benchmark timing is local-machine dependent and should be treated as a trend
  signal, not a release blocker, until CI variance is measured.
- Peak RSS is not captured; deterministic renderer budgets remain the primary
  memory guard.
- Visual correctness is still handled by fixture tests and later visual-diff
  workflow milestones.
