# Renderer Diagnostics Bundle 2026-06-25

## Summary

Native corpus runs can now emit compact diagnostic bundles for failing fixtures
through `summarize-fallbacks --diagnostics-dir <path>`. The command still emits
the normal fallback summary, and additionally writes one JSON bundle per fixture
whose native render returns fallback-required or error.

The bundle is intentionally safe by default:

- It does not include PDF bytes.
- It does not include rendered pixels.
- It does not include document-info fields such as title, author, subject, or
  producer.
- It does include the manifest entry, so path and manifest notes should still
  be reviewed before sharing outside the trust boundary.

## Format

Each bundle uses `schema_version: 1` and records:

| Field | Purpose |
| --- | --- |
| `backend` | Stable backend name, currently `rust-native`. |
| `path` | Normalized fixture path. |
| `manifest` | Matched corpus manifest entry, if available. |
| `privacy` | Explicit booleans for excluded private bytes, pixels, and document info. |
| `options` | Page index, max edge, background, output format, and timeout. |
| `metadata` | Safe page count and page sizes, or metadata error. |
| `stages` | Metadata timing plus render-pipeline timing, typed outcome, and stage hint. |
| `native_memory_diagnostics` | Renderer memory/cache budget snapshot. |

Stage hints are coarse until the backend exposes internal parser,
display-list, and raster timers. They still route common failures to useful
areas such as `parser-or-object`, `display-list-or-raster`,
`resource-decode-or-raster`, and `raster-or-memory-budget`.

## Smoke Run

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family presentation --max-edge 160 --diagnostics-dir target/diagnostics-0135 --output target/diagnostics-0135-summary.json
```

Fallback summary:

| Metric | Value |
| --- | ---: |
| Total presentation fixtures | 9 |
| Native rendered | 8 |
| Fallback required | 1 |
| Errors | 0 |
| Native pass rate | 0.889 |
| Diagnostic bundles | 1 |

Generated bundle:

```text
target/diagnostics-0135/0004-fixtures-generated-optional-content-ocmd-pdf.diagnostics.json
```

The bundle captures:

- `path`: `fixtures/generated/optional-content-ocmd.pdf`
- `category`: `graphics.optional-content`
- `stage_hint`: `display-list-or-raster`
- `metadata`: page count `1`, page size `100 x 80`
- `privacy.includes_pdf_bytes`: `false`
- `privacy.includes_rendered_pixels`: `false`
- `privacy.includes_document_info`: `false`

## Validation

- `cargo fmt --check`
- `cargo test -p pdfrust-cli diagnostic_bundles -- --nocapture`
- `cargo test -p pdfrust-cli fallback_summary_config_should_accept_family_filters -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family presentation --max-edge 160 --diagnostics-dir target/diagnostics-0135 --output target/diagnostics-0135-summary.json`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test --workspace --no-default-features`
