# Cross-Platform Rendering Determinism Gate 2026-06-25

Milestone: 0119.

## Implemented Slice

- Added target platform metadata to `benchmark-native`, `benchmark-pdfium`, and
  `visual-diff` JSON reports.
- Documented the cross-platform determinism policy and release-candidate gate
  expectations in `docs/policies/cross-platform-determinism.md`.
- Updated benchmark and visual-diff policy docs to make platform metadata part
  of the report contract.

## Local Platform

The 0119 gate was run locally on:

| Field | Value |
| --- | --- |
| `os` | `macos` |
| `arch` | `aarch64` |
| `family` | `unix` |
| `endian` | `little` |
| `pointer_width_bits` | `64` |

This is a valid local determinism baseline, not a full Linux/macOS matrix. The
policy requires additional platform artifacts before a native-only release
candidate can claim cross-platform coverage.

## Native Supported Gate

Artifact: `target/determinism-0119-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 0 | 0 |
| `office-export` | 24 | 24 | 0 | 0 |
| `form` | 14 | 14 | 0 | 0 |
| **Total** | **46** | **46** | **0** | **0** |

The supported-family native gate passes on the local macOS/aarch64 target.

## Benchmark Gate

Artifact: `target/determinism-0119-benchmark.json`

| Metric | Count |
| --- | ---: |
| Total fixtures | 106 |
| Native rendered | 99 |
| Fallback required | 6 |
| Errors | 1 |
| Budget failures | 7 |

Supported-family benchmark results:

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 21.411 | 47.434 | 0 |
| `office-export` | 24 | 24 | 4.526 | 37.466 | 0 |
| `form` | 14 | 14 | 18.078 | 78.073 | 0 |

The local supported families fit the configured benchmark budgets. Full-corpus
budget failures align with existing unsupported/error cases outside the
supported-family release surface.

## Visual-Diff Gate

Artifact: `target/determinism-0119-visual-diff.json`

Thresholds:

| Metric | Threshold |
| --- | ---: |
| `max_mean_abs_error` | 2.0 |
| `max_p95_channel_delta` | 16 |
| `max_changed_ratio` | 0.05 |

Full-corpus result:

| Metric | Count |
| --- | ---: |
| Total fixtures | 106 |
| Exact | 35 |
| Accepted drift | 22 |
| Blockers | 42 |
| Native errors | 6 |
| PDFium errors | 0 |
| Both errors | 1 |

Supported-family fidelity:

| Family | Total | Exact | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 4 | 2 | 2 | 0 |
| `office-export` | 24 | 0 | 4 | 20 | 0 |
| `form` | 14 | 0 | 3 | 11 | 0 |

Primary blocker subsystems:

| Subsystem | Blockers | Example fixtures |
| --- | ---: | --- |
| `text-fonts` | 20 | `arabic-shaped-text.pdf`, `cff-fontfile3-text.pdf`, `cid-font-text.pdf` |
| `annotations-forms` | 10 | `acroform-checkbox-missing-appearance.pdf`, `acroform-checkbox.pdf`, `acroform-choice-missing-appearance.pdf` |
| `images-color` | 3 | `cmyk-image.pdf`, `icc-cmyk-image.pdf`, `scanned-page.pdf` |
| `page-geometry` | 3 | `multi-page-report.pdf`, `rotated-office-export.pdf`, `user-unit-page.pdf` |
| `rendering-core` | 3 | `devicen-spot-color.pdf`, `office-table.pdf`, `separation-spot-color.pdf` |
| `document-structure` | 1 | `hybrid-reference.pdf` |
| `transparency` | 1 | `transparency-alpha.pdf` |
| `vector-graphics` | 1 | `vector-stress.pdf` |

## Decision

The local macOS/aarch64 target passes native supported-family execution and
benchmark budgets, but visual fidelity remains blocked for supported families.
0119 therefore strengthens evidence capture and release gating rather than
declaring cross-platform native-only readiness.

Linux and other supported target artifacts are required before a native-only
release candidate can pass the cross-platform determinism gate.

## Validation Commands

```text
cargo fmt --check
cargo check --workspace
cargo check --workspace --all-features
cargo test -p ferrugo-cli benchmark_native_should_group_results_and_budget_failures -- --nocapture
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/determinism-0119-benchmark.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/determinism-0119-supported-gate.json
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/determinism-0119-visual-diff.json
```
