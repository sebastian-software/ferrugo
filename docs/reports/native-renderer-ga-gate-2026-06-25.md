# Native Renderer General Availability Gate

Date: 2026-06-25
Milestone: 0100

## Decision

The Rust-native renderer is not ready for broad general availability as a
PDFium replacement across the targeted typical-document surface.

It is ready for PDFium-free technical operation on the supported-family native
gate, where `browser-print`, `office-export`, and `form` render without native
errors or PDFium fallback. The stricter PDFium visual baseline still reports
material fidelity blockers in those same families, so the release decision is a
conditional no-go for visual GA and a go only for native-only supported-subset
execution under documented quality expectations.

Normal supported-subset gates should remain native-only. PDFium remains a
maintainer comparison oracle, an explicit emergency fallback, and a fallback for
unsupported categories. It should not be packaged or enabled by default for
normal native-only execution.

## Native-Only Supported Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/ga-0100-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 5 | 5 | 0 | 0 |
| `office-export` | 14 | 14 | 0 | 0 |
| `form` | 11 | 11 | 0 | 0 |
| **Supported gate total** | **30** | **30** | **0** | **0** |

This confirms that the supported-family execution path can run without PDFium
as a normal dependency.

## Renderer Benchmark Evidence

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/ga-0100-benchmark-native.json
```

Summary:

| Metric | Count |
| --- | ---: |
| Total fixtures | 75 |
| Native rendered | 69 |
| Fallback required | 5 |
| Errors | 1 |
| Budget failures | 6 |

Supported-family performance:

| Family | Total | Mean ms | Max ms | Output bytes | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 5 | 22.925 | 45.786 | 297600 | 0 |
| `office-export` | 14 | 6.843 | 36.449 | 844160 | 0 |
| `form` | 11 | 5.202 | 14.397 | 435200 | 0 |

The supported-family benchmark slice fits the configured time and output-size
budgets. The full-corpus budget failures align with unsupported or not-yet-GA
families and should stay out of the supported GA claim.

## PDFium Visual Baseline

Command:

```sh
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 120 \
  --output target/ga-0100-visual-diff.json
```

Thresholds:

| Metric | Threshold |
| --- | ---: |
| `max_mean_abs_error` | 2 |
| `max_p95_channel_delta` | 16 |
| `max_changed_ratio` | 0.05 |

Full-corpus result:

| Metric | Count |
| --- | ---: |
| Total fixtures | 75 |
| Exact | 26 |
| Accepted drift | 13 |
| Blockers | 30 |
| Native errors | 5 |
| PDFium errors | 0 |
| Both errors | 1 |

Supported-family fidelity:

| Family | Total | Exact | Accepted drift | Blockers | Native errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 5 | 2 | 2 | 1 | 0 |
| `office-export` | 14 | 0 | 2 | 12 | 0 |
| `form` | 11 | 0 | 1 | 10 | 0 |

Primary blocker clusters:

| Cluster | Count | Examples |
| --- | ---: | --- |
| `form` / `annotations-forms` | 10 | checkbox, radio, choice, signature placeholder, missing appearance synthesis |
| `office-export` / `text-fonts` | 9 | embedded fonts, spacing, shaped RTL text, ToUnicode, Type3 barcode |
| `office-export` / `page-geometry` | 2 | multi-page report, rotated office export |
| `office-export` / `rendering-core` | 1 | office table |
| `browser-print` / `page-geometry` | 1 | user-unit page |

These blockers prevent a broad GA declaration. The next milestones should
prioritize text/font fallback and form appearance fidelity before revisiting a
visual GA claim.

## Safety And Packaging Evidence

Fuzz smoke targets:

| Target | Smoke cases |
| --- | ---: |
| `primitive_parse` | 165 |
| `xref_load` | 154 |
| `stream_decode` | 154 |
| `content_tokenize` | 165 |
| `render_setup` | 165 |

Package dry-runs:

| Package | Raw size | Compressed size |
| --- | ---: | ---: |
| `ferrugo-syntax` | 27.1 KiB | 6.2 KiB |
| `ferrugo-thumbnail` | 15.5 KiB | 4.5 KiB |

`ferrugo-cli` packaging remains intentionally release-order bound until the
internal crates are published in dependency order, as documented in
`docs/packaging.md`.

## Remaining Unsupported Surface

The current native fallback surface remains:

| Category | Status |
| --- | --- |
| `graphics.optional-content` | OCMD policy support still required. |
| `graphics.pattern-shading` | Mesh shading support still required. |
| `image.filter` | CCITT, JBIG2, and JPX policy/support still required. |
| `encrypted` | Explicit native error class, not a PDFium fallback category. |

## Rollback And Fallback Policy

| Path | Decision |
| --- | --- |
| Normal native-only supported-subset gates | Keep PDFium disabled. |
| Visual GA claim | Do not ship yet; visual blockers remain in supported families. |
| `--allow-pdfium-fallback` | Keep explicit only for maintainer probes, unsupported categories, and emergency use. |
| `render-pdfium`, `compare-metadata`, `benchmark-pdfium`, `visual-diff` | Keep as maintainer comparison tools. |
| Default packaging | Keep PDFium out of default/native-only builds. |

## Post-GA Maintenance Backlog

- 0101: system and common-font fallback policy for visible text fidelity.
- 0102: CFF Type 1 charstring hardening for embedded-font coverage.
- 0103: OpenType layout feature coverage for shaping and spacing drift.
- 0104: advanced CMap encodings and identity mapping.
- 0105-0110: color, ICC, pattern, shading, transparency, and overprint fidelity.
- 0111-0113: dynamic forms, signature appearances, embedded files, and portfolio
  boundaries.
- 0120: PDFium-free maintenance gate and deletion backlog after fidelity gaps
  are remeasured.

## Validation

- `cargo fmt --check`
- `cargo check --workspace --no-default-features`
- `cargo test --workspace --no-default-features`
- `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/ga-0100-supported-gate.json`
- `cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/ga-0100-benchmark-native.json`
- `cargo run -p ferrugo-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 120 --output target/ga-0100-visual-diff.json`
- `cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke`
- `cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke`
- `cargo package -p ferrugo-syntax --allow-dirty --no-verify`
- `cargo package -p ferrugo-thumbnail --allow-dirty --no-verify`
- `cargo test -p ferrugo-cli --features pdfium`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
