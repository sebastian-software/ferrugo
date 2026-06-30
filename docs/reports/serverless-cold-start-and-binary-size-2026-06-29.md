# Serverless Cold Start And Binary Size 2026-06-29

Milestone 0197 adds a native-only serverless build and measurement gate for
short-lived thumbnail workers. The goal is to keep the PDFium-free deployment
path small, reproducible, and fast enough to start new worker processes without
shipping a native PDFium runtime.

## Implementation

- Added Cargo profile `serverless`.
- Added `scripts/measure_serverless_profile.sh`.
- The script builds `ferrugo-cli` with `--profile serverless` and
  `--no-default-features`.
- The script checks the `ferrugo-cli` package file list for PDFium/runtime
  native assets.
- Startup is measured with a fresh `ferrugo-cli --help` process per sample.
- First render is measured with a fresh `render-native` process per sample.

The `serverless` profile inherits release mode and sets:

| Setting | Value |
| --- | --- |
| `lto` | `thin` |
| `codegen-units` | `1` |
| `opt-level` | `z` |
| `panic` | `abort` |
| `strip` | `symbols` |

## Budgets

Default local budgets:

| Budget | Limit |
| --- | ---: |
| Binary size | 8 MiB |
| Startup p95 | 500 ms |
| First-render p95 | 250 ms |
| Render output | 1 MiB |

The startup budget intentionally allows true cold-process variance. The
first-render budget remains tighter because the measured fixture is small and
the render path should not need PDFium or external assets.

## Measurement

Command:

```sh
bash scripts/measure_serverless_profile.sh
```

Artifact:

- `target/serverless-profile-0197.json`

Result:

| Binary bytes | Render output bytes | Budget failures |
| ---: | ---: | ---: |
| 1,017,344 | 54,553 | 0 |

Startup latency:

| Min ms | Mean ms | P50 ms | P95 ms | Max ms |
| ---: | ---: | ---: | ---: | ---: |
| 2.897 | 31.788 | 3.208 | 203.693 | 203.693 |

First-render latency:

| Min ms | Mean ms | P50 ms | P95 ms | Max ms |
| ---: | ---: | ---: | ---: | ---: |
| 3.845 | 4.577 | 4.599 | 5.188 | 5.188 |

## Package Inspection

The same script writes and inspects:

- `target/serverless-profile-ferrugo-cli-package-files.txt`

No PDFium runtime asset, dynamic library, static native archive, framework, or
`FERRUGO_PDFIUM_LIBRARY` packaging hook was found in the native-only CLI package
file list.

## Follow-Up List

- Re-run this gate on Linux CI before making cross-platform serverless budget
  claims.
- Track codec dependency changes through this script before promoting optional
  heavyweight features into the default server profile.
- Keep WASM/mobile size gates separate; they are secondary profile checks, not
  blockers for this server-side path.

## Validation Commands

```text
bash scripts/measure_serverless_profile.sh
cargo fmt --check
git diff --check -- Cargo.toml scripts/measure_serverless_profile.sh docs/packaging.md docs/benchmarks.md docs/reports/serverless-cold-start-and-binary-size-2026-06-29.md
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
