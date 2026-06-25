# PDFium-Free Maintenance Gate 2026-06-25

Milestone: 0120.

## Decision

Normal supported-document rendering remains PDFium-free by default. PDFium
should stay as explicit maintainer-only oracle and comparison tooling until the
remaining unsupported buckets and visual blockers have native coverage or a
documented non-PDFium oracle strategy.

No PDFium runtime dependency is present in the native-only `pdfrust-cli`
dependency graph.

## Dependency Graph Audit

Native-only CLI graph:

```text
cargo tree -p pdfrust-cli --no-default-features
```

Result: no `pdfrust-pdfium` edge. The graph contains `pdfrust-native`,
`pdfrust-content`, `pdfrust-object`, `pdfrust-render`, `pdfrust-syntax`,
`pdfrust-thumbnail`, and pure Rust/image/font dependencies.

PDFium-enabled CLI graph:

```text
cargo tree -p pdfrust-cli --features pdfium
```

Result: adds `pdfrust-pdfium` only through the explicit `pdfium` feature.

## Native-Only Supported Gate

Artifact: `target/maintenance-0120-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 8 | 8 | 0 | 0 |
| `office-export` | 24 | 24 | 0 | 0 |
| `form` | 14 | 14 | 0 | 0 |
| **Total** | **46** | **46** | **0** | **0** |

The supported-document release surface renders natively without PDFium fallback
or errors.

## Native Benchmark Evidence

Artifact: `target/maintenance-0120-benchmark.json`

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
| `browser-print` | 8 | 8 | 21.203 | 46.250 | 0 |
| `office-export` | 24 | 24 | 4.391 | 36.607 | 0 |
| `form` | 14 | 14 | 7.742 | 35.582 | 0 |

## Package Validation

Package file listing:

```text
cargo package -p pdfrust-cli --allow-dirty --no-verify --list
```

Result:

```text
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
src/main.rs
```

`cargo package -p pdfrust-cli --allow-dirty --no-verify` fails because
`pdfrust-native` is not yet available from crates.io. The same failure occurs
after allowing network access and is a release-order blocker, not a PDFium
dependency leak.

Leaf package dry-runs pass:

| Package | Result |
| --- | --- |
| `pdfrust-syntax` | packaged 5 files, 27.1 KiB |
| `pdfrust-thumbnail` | packaged 5 files, 16.9 KiB |

## Maintainer PDFium Evidence

Artifact: `target/maintenance-0120-pdfium-benchmark.json`

PDFium-enabled benchmark summary:

| Metric | Count |
| --- | ---: |
| Total fixtures | 106 |
| PDFium rendered | 105 |
| Fallback required | 0 |
| Errors | 1 |
| Budget failures | 1 |

Supported-family PDFium benchmark results have 0 errors and 0 budget failures.
This remains useful as maintainer oracle evidence, but it is not part of normal
native-only package validation.

## Backlog

The deletion and retention backlog is tracked in
`docs/backlogs/pdfium-free-maintenance-backlog.md`.

Immediate decision:

- Keep `pdfrust-pdfium` and PDFium CLI commands as explicit maintainer tooling.
- Keep PDFium fallback opt-in only for unsupported-category probes.
- Do not bundle or enable PDFium in native-only package/deployment flows.
- Schedule small reversible deletions for old aliases and env-driven fallback
  behavior after the 0142 tooling-quarantine milestone.

## Validation Commands

```text
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo tree -p pdfrust-cli --no-default-features
cargo tree -p pdfrust-cli --features pdfium
cargo package -p pdfrust-cli --allow-dirty --no-verify --list
cargo package -p pdfrust-cli --allow-dirty --no-verify
cargo package -p pdfrust-syntax --allow-dirty --no-verify
cargo package -p pdfrust-thumbnail --allow-dirty --no-verify
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/maintenance-0120-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/maintenance-0120-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- benchmark-pdfium fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/maintenance-0120-pdfium-benchmark.json
cargo test -p pdfrust-cli --features pdfium pdfium -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- render-native fixtures/generated/office-table.pdf --max-edge 160 --output target/maintenance-0120-office-table.png
```
