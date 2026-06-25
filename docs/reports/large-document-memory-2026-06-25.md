# Large Document Memory 2026-06-25

Milestone: 0095.

## Implemented Slice

- Added `DEFAULT_TOTAL_IMAGE_BYTES_LIMIT` as a page-level decoded image budget.
- Extended `DisplayListOptions` with `max_total_image_bytes`.
- Enforced the total resident decoded image budget while resolving image
  XObject resources.
- Mapped total image budget failures to the existing stable
  `renderer.memory-budget` bucket.
- Exposed `max_total_image_bytes`, `spooling_enabled`, and `max_spool_bytes`
  through `NativeMemoryDiagnostics` and CLI `rust_native_memory` JSON.
- Kept temporary spooling disabled by default with an explicit `0` byte budget.

## Policy

`docs/policies/renderer-memory-budgets.md` now documents the page-level image
budget and the disabled-by-default spooling policy. This keeps document-derived
intermediates in memory unless a future explicit opt-in policy defines storage,
limits, cleanup, and privacy behavior.

## Budget Values

| Budget | Value |
| --- | ---: |
| Per-image decoded bytes | 33,554,432 |
| Total page image bytes | 134,217,728 |
| Spooling enabled | false |
| Max spool bytes | 0 |

## Targeted Tests

```text
cargo test -p pdfrust-render total_image_byte_budget -- --nocapture
cargo test -p pdfrust-native memory_diagnostics -- --nocapture
cargo test -p pdfrust-cli comparison_json_should_include_match_status -- --nocapture
```

The renderer test builds two individually valid image XObjects whose combined
decoded sample bytes exceed a tight page image budget. It returns
`ImageResourceBytesOverflow` before storing the second image in the resource
map.

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/large-document-memory-benchmark-0095.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 75 | 69 | 5 | 1 | 7 |

Selected families:

| Family | Total | Native rendered | Fallback required | Errors | Budget failures | Mean ms | Max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| mixed-layout | 15 | 14 | 0 | 1 | 1 | `32.910` | `177.979` |
| office-export | 14 | 14 | 0 | 0 | 0 | `29.168` | `90.245` |
| scan | 13 | 10 | 3 | 0 | 3 | `11.739` | `85.899` |

## Validation

```text
cargo fmt
cargo fmt --check
cargo check --workspace --no-default-features
cargo test -p pdfrust-render total_image_byte_budget -- --nocapture
cargo test -p pdfrust-native memory_diagnostics -- --nocapture
cargo test -p pdfrust-cli comparison_json_should_include_match_status -- --nocapture
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/large-document-memory-benchmark-0095.json
```

All listed commands completed successfully.

## Remaining Limits

- This slice adds a deterministic decoded-image resident budget; it does not
  introduce disk spooling.
- Font-program and form-resource lifetime remains scoped to resource
  construction and render calls rather than a persistent cross-document cache.
- Operating-system RSS measurement remains outside the deterministic renderer
  budget model.
